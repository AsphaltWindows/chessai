#ifndef CHESS_BOARD_ACTIONS_H
#define CHESS_BOARD_ACTIONS_H

#include "position.h"
#include "move.h"

#define LONG_CASTLE_BLANK 0xFFF000F0
#define SHORT_CASTLE_BLANK 0x0000FFFF
#define WHITE_LONG_CASTLE_FILL 0x00004600
#define WHITE_SHORT_CASTLE_FILL 0x06400000
#define BLACK_LONG_CASTLE_FILL 0x0000CE00
#define BLACK_SHORT_CASTLE_FILL 0x0EC00000

void starting_position(
        position_t * to_change);

void do_move(
        position_t const * from,
        move_t move,
        position_t * to);

move_t * legal_moves(
        position_t * pos_ptr,
        size_t * outsize);

#endif
