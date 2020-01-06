#ifndef POSITION_H
#define POSITION_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#define BITBOARD_NUM 14

#define WHITE_WIN 0
#define BLACK_WIN 1
#define DRAW 2
#define INCOMPLETE 3

typedef int g_state_t;

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

g_state_t position_state(
        const position_t * const pos);

bool is_pawn_move(
        move_t);

bool is_capture(
        move_t);

#endif

