#include "game.h"

#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "utils.h"

#define FREQ_HASHTABLE_SIZE 307

static bool add_p_freq(
        game_t * game,
        const position_t * const pos_ptr);
static int get_p_freq(
        const game_t * const game,
        const position_t * const pos);

game_t * create_game()
{
    game_t * res;

    if (!(res = malloc(sizeof(game_t)))) {
        printf("Failed to allocate memory for chess game\n.");
        return NULL;
    }

    memset(res, 0, sizeof(game_t));

    if (!(res->freq_hashtable = malloc(sizeof(p_freq_t *) * FREQ_HASHTABLE_SIZE))) {
        printf("Failed to allocate memory for frequency hash table.\n");
        free_game(res);
        return NULL;
    }

    memset(res->freq_hashtable, 0, sizeof(p_freq_t *) * FREQ_HASHTABLE_SIZE);

    if (!(res->pos = malloc(sizeof(p_hist_t)))) {
        printf("Failed to allocate memory for game position.\n");
        free_game(res);
        return NULL;
    }

    memset(res->pos, 0, sizeof(p_hist_t));
    starting_position(&(res->pos->pos));

    if (!(add_p_freq(
            res,
            (const position_t * const) &(res->pos->pos)))) {
        printf("Failed to add new position to position frequency map.\n");
        free_game(res);
        return NULL;
    }

    return res;
}

static bool add_p_freq(
        game_t * game,
        const position_t * const pos_ptr)
{
    uint64_t idx;
    p_freq_t * found_freq;
    p_freq_t * start_freq;

    idx = position_hash_index(
            (const position_t * const) pos_ptr,
            FREQ_HASHTABLE_SIZE);

    found_freq = game->freq_hashtable[idx];

    while (found_freq) {

        if (!memcmp(
                (const void *) found_freq->pos,
                (const void *) pos_ptr,
                sizeof(position_t))) {
            found_freq->freq++;
            return true;
        }
        else {
            found_freq = found_freq->next;
        }
    }

    if (!(start_freq = malloc(sizeof(p_freq_t)))) {
        printf("Failed to allocate memory for position frequency.\n");
        return false;
    }

    memset(start_freq, 0, sizeof(p_freq_t));
    start_freq->pos = (position_t *) pos_ptr;
    start_freq->freq = 1;
    start_freq->next = game->freq_hashtable[idx];
    game->freq_hashtable[idx] = start_freq;

    return true;
}

void free_game(
        game_t * game)
{
    p_freq_t * freq_at, * freq_next;
    p_hist_t * hist_at, * hist_prev;

    if (!game) {
        return;
    }

    if (game->freq_hashtable) {

        for (int i = 0; i < FREQ_HASHTABLE_SIZE; ++i) {
            freq_at = game->freq_hashtable[i];

            while (freq_at) {
                freq_next = freq_at->next;
                free(freq_at);
                freq_at = freq_next;
            }
        }

        free(game->freq_hashtable);
        game->freq_hashtable = NULL;
    }

    if (game->pos) {
        hist_at = game->pos;

        while(hist_at) {
            hist_prev = hist_at->prev;
            free(hist_at);
            hist_at = hist_prev;
        }

        game->pos = NULL; }

    free(game);
}

g_state_t game_state(
        const game_t * const game)
{
    g_state_t pos_state;

    pos_state = position_state(
            (const position_t * const) &(game->pos->pos));

    if (pos_state != INCOMPLETE) {
        return pos_state;
    }
    else if (game->fiftyMoveRuleCounter == 100) {
        return DRAW;
    }
    else if (get_p_freq(
            game,
            (const position_t * const) &(game->pos->pos)) >= 3) {
        return DRAW;
    }
    else {
        return INCOMPLETE;
    }
}


static int get_p_freq(
        const game_t * const game,
        const position_t * const pos_ptr)
{
    uint64_t idx;
    p_freq_t *found_freq;

    idx = position_hash_index(
            pos_ptr,
            FREQ_HASHTABLE_SIZE);

    found_freq = game->freq_hashtable[idx];

    while (found_freq) {

        if (!memcmp(
                (const void *) found_freq->pos,
                (const void *) pos_ptr,
                sizeof(position_t))) {
            return found_freq->freq;
        }
        else {
            found_freq = found_freq->next;
        }
    }

    return 0;
}


bool play_move(
        game_t * game,
        move_t move)
{
    p_hist_t * new_pos;

    if (!(new_pos = malloc(sizeof(p_hist_t)))) {
        printf("Failed to allocate memory for new game position.\n");
        return false;
    }

    memset(new_pos, 0, sizeof(p_hist_t));

    do_move(
            &(game->pos->pos),
            move,
            &(new_pos->pos));

    if (!(add_p_freq(
            game,
            (const position_t * const) &(new_pos->pos)))) {
        printf("Failed to add new position to position frequency map.\n");
        free(new_pos);
        return false;
    }

    if (is_capture(move) || is_pawn_move(move)) {
        game->fiftyMoveRuleCounter = 0;
    }
    else {
        game->fiftyMoveRuleCounter++;
    }

    game->pos->move = move;
    new_pos->prev = game->pos;
    game->pos = new_pos;

    return true;
}

