#ifndef POSITION_H
#define POSITION_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#include "types.h"
#include "constants.h"
#include "move.h"

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

inline piece_t piece_at(
        position_t const * pos,
        square_t s);

inline void set_piece_at(
        position_t * pos,
        piece_t p,
        square_t s);

inline void disable_white_long_castle(
        position_t * pos);

inline void disable_white_short_castle(
        position_t * pos);

inline void disable_white_all_castle(
        position_t *pos);

inline void disable_black_long_castle(
        position_t * pos);

inline void disable_black_short_castle(
        position_t * pos);

inline void disable_black_all_castle(
        position_t *pos);

inline void change_turn(
        position_t * pos);

inline void set_enpassant(
        position_t * pos,
        file_t f);

inline void reset_enpassant(
        position_t * pos);

inline int white_long_castle_flag(
        position_t * pos);

inline int white_short_castle_flag(
        position_t * pos);

inline int black_long_castle_flag(
        position_t * pos);

inline int black_short_castle_flag(
        position_t * pos);


#endif

