#ifndef CHESSAI_HARNESS_PLAYER_H
#define CHESSAI_HARNESS_PLAYER_H

#include "types.h"
#include "model.h"

#include <stdint.h>

#define WLD_SCORE "wld_score"
#define WLD_SCORE_JITTER_10 "wld_score_jitter10"
#define WLD_SCORE_JITTER_20 "wld_score_jitter20"
#define WLD_WIN_DRAW "wld_wd"
#define RANDOM_SELECT "rand"

player_t * player(
        em_t * eval_model,
        const char * compare_type,
        color_t player_color);

void free_player(
        player_t * player);

uint32_t select_move(
        const player_t * player,
        const uint8_t * const * positions,
        size_t score_num);

void switch_color(
        player_t * player);

void set_color(
        player_t * player,
        color_t color);

#endif

