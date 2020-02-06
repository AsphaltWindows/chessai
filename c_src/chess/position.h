#ifndef CHESS_POSITION_H
#define CHESS_POSITION_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#include "types.h"
#include "constants.h"
#include "move.h"

#define RS_WHITE_LONG_CASTLE 7
#define RS_WHITE_SHORT_CASTLE 6
#define RS_BLACK_LONG_CASTLE 5
#define RS_BLACK_SHORT_CASTLE 4
#define RS_CASTLE_FLAG 4
#define RS_EN_PASSANT_FLAG 8

#define SIZE_CASTLE_FLAG 4
#define SIZE_EN_PASSANT_FLAG 4

#define MASK_CASTLE_FLAG 15
#define MASK_EN_PASSANT_FLAG 15

g_state_t position_state(
        const position_t * const pos);

inline piece_t piece_at(
        position_t const * pos,
        square_t s);

inline void set_piece_at(
        position_t * pos,
        piece_t p,
        square_t s);

inline void remove_piece_at(
        position_t * pos,
        piece_t p,
        square_t s);

//inline void apply_flag_mask(
//        position_t * pos,
//        int disable_castle,
//        int en_passant);
//


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

