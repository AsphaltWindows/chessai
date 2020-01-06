#include <stdio.h>
#include <math.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

#include "chess_context.h"

#include "utils.h"
#include "position.h"

static bool is_prime(
        uint64_t num);
static uint64_t smallest_prime_greater_than(
        uint64_t num);
static void cleanup_context_cache(
        chess_ctx_t * ctx);
static void cleanup_bucket(
        chess_ctx_t * ctx,
        n_bucket_t * bucket);

chess_ctx_t * create_context(
        uint64_t pos_limit)
{
    chess_ctx_t * res;
    uint64_t prime;

    if (!(res = malloc(sizeof(chess_ctx_t)))) {
        printf("Failed to allocate memory for chess context.\n");
        return NULL;
    }

    memset(res, 0, sizeof(chess_ctx_t));
    prime = smallest_prime_greater_than(pos_limit);

    res->pos_limit = pos_limit;
    res->prime = prime;

    if (!(res->node_hashtable = malloc(sizeof(n_bucket_t *) * prime))) {
        printf("Failed to allocate hash table array.\n");
        free_context(res);
        return NULL;
    }

    memset(res->node_hashtable, 0, sizeof(n_bucket_t *) * prime);

    if (!(res->bucket_space = malloc(sizeof(n_bucket_t) * pos_limit))) {
        printf("Failed to allocate memory for position buckets.\n");
        free_context(res);
        return NULL;
    }

    memset(res->bucket_space, 0, sizeof(n_bucket_t) * pos_limit);

    for (uint64_t i = 0; i < pos_limit - 1; ++i) {
        res->bucket_space[i].next = &(res->bucket_space[i + 1]);
    }

    res->free_bucket = &(res->bucket_space[0]);

    return res;
}

static uint64_t smallest_prime_greater_than(
        uint64_t num)
{
    uint64_t at = num;

    while (!is_prime(at)) {
        ++at;
    }

    return at;
}

static bool is_prime(
        uint64_t num)
{
    uint32_t upto;
    uint32_t at = 2;

    upto = (uint32_t) (sqrt((double) num)) + 1;

    while (at < upto) {

        if (num % at == 0) {
            return false;
        }

        ++at;
    }

    return true;
}

void free_context(
        chess_ctx_t * ctx)
{

    if  (!ctx) {
        return;
    }

    if (ctx->node_hashtable) {
        free(ctx->node_hashtable);
    }

    if (ctx->bucket_space) {
        free(ctx->bucket_space);
    }

    free(ctx);
}

node_t * position_node(
        chess_ctx_t * ctx,
        position_t * pos_ptr,
        gid_t game_id)
{
    move_t * moves;
    n_bucket_t * found_bucket, * new_bucket;
    size_t move_num;

    uint64_t idx = position_hash_index(
            (const position_t *) pos_ptr,
            ctx->prime);

    found_bucket = ctx->node_hashtable[idx];

    while (found_bucket) {

        if (!memcmp((const void *) (&(found_bucket->node.pos)), (const void *) pos_ptr, sizeof(position_t))) {
            found_bucket->node.gid = game_id;
            found_bucket->node.hits_num++;
            return &(found_bucket->node);
        }

        found_bucket = found_bucket->next;
    }

    if (ctx->pos_limit <= ctx->num_pos) {
        cleanup_context_cache(ctx);
    }

    if (!(moves = legal_moves(pos_ptr, &move_num))) {
        printf("Failed to generate legal moves for position.\n");
        return NULL;
    }

    new_bucket = ctx->free_bucket;
    ctx->free_bucket = new_bucket->next;
    new_bucket->next = found_bucket;

    memcpy(&(new_bucket->node.pos), pos_ptr, sizeof(position_t));
    new_bucket->node.hits_num = 1;
    new_bucket->node.moves = moves;
    new_bucket->node.move_num = move_num;
    new_bucket->node.gid = game_id;
    ctx->node_hashtable[idx] = new_bucket;
    ctx->num_pos++;

    return &(new_bucket->node);
}

static void cleanup_context_cache(
        chess_ctx_t * ctx)
{
    uint64_t total_num = 0;
    uint64_t total_hits = 0;
    uint64_t average;
    n_bucket_t * bucket, * prev_bucket;

    for (uint64_t at = 0; at < ctx->prime; ++at) {
        bucket = ctx->node_hashtable[at];

        while (bucket) {
            total_num += 1;
            total_hits += bucket->node.hits_num;
            bucket = bucket->next;
        }
    }

    average = total_num ? total_hits / total_num : 0;

    for (uint64_t at = 0; at < ctx->prime; ++at) {
        bucket = ctx->node_hashtable[at];
        prev_bucket = NULL;

        while (bucket) {

            if (bucket->node.hits_num > average) {
                bucket->node.hits_num -= average;
                prev_bucket = bucket;
                bucket = bucket->next;
            }
            else {
                if (!prev_bucket) {
                    ctx->node_hashtable[at] = bucket->next;
                    cleanup_bucket(
                            ctx,
                            bucket);
                    bucket = ctx->node_hashtable[at];
                }
                else {
                    prev_bucket->next = bucket->next;
                    cleanup_bucket(
                            ctx,
                            bucket);
                    bucket = prev_bucket->next;
                }
                ctx->num_pos--;
            }
        }
    }
}

static void cleanup_bucket(
        chess_ctx_t * ctx,
        n_bucket_t * bucket)
{
    free(bucket->node.moves);
    memset(&(bucket->node), 0, sizeof(node_t));
    bucket->next = ctx->free_bucket;
    ctx->free_bucket = bucket;
}

node_t * new_game(
        chess_ctx_t * ctx,
        gid_t game_id)
{
    position_t pos;
    starting_position(&pos);

    return position_node(
            ctx,
            &pos,
            game_id);
}



