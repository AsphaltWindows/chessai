#ifndef POSITION_H
#define POSITION_H

#include <stddef.h>
#include <stdint.h>

#define BITBOARD_NUM 14

typedef uint32_t move_t;

typedef struct chess_position {
    uint64_t bb[BITBOARD_NUM];
} position_t;

void starting_position(
        position_t * to_change);

void do_move(
        position_t * from,
        move_t move,
        position_t * to);

move_t * legal_moves(
        position_t * pos_ptr,
        size_t *outsize);

#endif

