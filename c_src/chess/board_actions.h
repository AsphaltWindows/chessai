#ifndef BOARD_ACTIONS_H
#define BOARD_ACTIONS_H

#include "position.h"
#include "move.h"

void starting_position(
        position_t * to_change);

void do_move(
        position_t const * from,
        move_t move,
        position_t * to);

void undo_move(
        position_t const * from,
        move_t move,
        position_t * to);

move_t * legal_moves(
        position_t * pos_ptr,
        size_t * outsize);

#endif
