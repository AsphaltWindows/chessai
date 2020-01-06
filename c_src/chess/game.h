#ifndef GAME_H
#define GAME_H

#include "position.h"
#include "stdint.h"

typedef struct position_history {
    struct position_history * prev;
    position_t pos;
    move_t move;
} p_hist_t;

typedef struct position_frequency {
    struct position_frequency * next;
    position_t * pos;
    uint8_t freq;
} p_freq_t;

typedef struct chess_game {
    p_hist_t * pos;
    p_freq_t ** freq_hashtable;
    uint32_t fiftyMoveRuleCounter;
} game_t;

game_t * create_game();

void free_game(
        game_t * game);

g_state_t game_state(
        const game_t * const game);

bool play_move(
        game_t * game,
        move_t move);

#endif //c_src_game_h
