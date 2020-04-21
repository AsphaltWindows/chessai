#ifndef CHESSAI_HARNESS_PLAYER_H
#define CHESSAI_HARNESS_PLAYER_H

#include "types.h"
#include "model.h"

#include <stdint.h>

#define WLD_SCORE "wld_score"
#define WLD_WIN_DRAW "wld_wd"

player_t * create_player(
        em_t * eval_model,
        const char * compare_type,
        color_t player_color);

void free_player(
        player_t * player);

uint32_t select_move(
        const player_t * player,
        const double * const * move_scores,
        size_t score_num);

void switch_color(
        player_t * player);


#endif
