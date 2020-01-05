#ifndef CHESS_CONTEXT_H
#define CHESS_CONTEXT_H

#include <stddef.h>
#include <stdint.h>

#include "position.h"

#define MAX_PIECE_NUM 32

typedef uint32_t gid_t;

typedef struct chess_node {
    position_t pos;
    move_t * moves;
    gid_t gid; // to determine if position is relevant for current game
    uint32_t hits_num;
    uint32_t move_num; // -1 if children not yet calculated
} node_t;

typedef struct node_bucket {
    struct node_bucket * next;
    node_t node;
} n_bucket_t;

typedef struct chess_context {
    n_bucket_t ** node_hashtable;
    uint64_t num_pos;
    uint64_t pos_limit;
    uint64_t prime;
    n_bucket_t * bucket_space;
    n_bucket_t * free_bucket;
} chess_ctx_t;

chess_ctx_t * create_context(
        uint64_t pos_limit);

void free_context(
        chess_ctx_t * ctx);

node_t * position_node(
        chess_ctx_t * ctx,
        position_t * pos_ptr,
        gid_t game_id);

node_t * new_game(
        chess_ctx_t * ctx,
        gid_t game_id);


#endif //CHESS_CONTEXT_H
