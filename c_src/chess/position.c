#include "position.h"

/** Bitboard Indices **/

inline piece_t piece_at(
        position_t const * pos,
        square_t s)
{
    return (piece_t) ((pos->bb[square_rank(s)] << (square_file(s) * 4)) & 15);
}


inline void set_piece_at(
        position_t * pos,
        piece_t p,
        square_t s)
{
    file_t f = square_file(s);
    rank_t r = square_rank(s);

    pos->bb[f] = pos->bb[r] | (15 << (f * 4)) & (p << (f * 4));
    return;
}

inline void disable_white_long_castle(
        position_t * pos)
{
    pos->bb[8] &= 0xFFFFFF7F;
}

inline void disable_white_short_castle(
        position_t * pos)
{
    pos->bb[8] &= 0xFFFFFFBF;
}

inline void disable_white_all_castle(
        position_t *pos)
{
    pos->bb[8] &= 0xFFFFFF3F;
}

inline void disable_black_long_castle(
        position_t * pos)
{
    pos->bb[8] &= 0xFFFFFFDF;
}

inline void disable_black_short_castle(
        position_t * pos)
{
    pos->bb[8] &= 0xFFFFFFEF;
}

inline void disable_black_all_castle(
        position_t *pos)
{
    pos->bb[8] &= 0xFFFFFFCF;
}

inline void change_turn(
        position_t * pos)
{
    pos->bb[8] ^= 1;
    return;
}

inline void set_enpassant(
        position_t * pos,
        file_t f)
{
    pos->bb[8] = pos->bb[8] & 0xFFFFFFF1 | (f << 1);
}

inline void reset_enpassant(
        position_t * pos)
{
    pos->bb[8] = pos->bb[8] & 0xFFFFFFF1;
}

inline int white_long_castle_flag(
        position_t * pos)
{
    return pos->bb[8] & 0X00000080;
}

inline int white_short_castle_flag(
        position_t * pos)
{
    return pos->bb[8] & 0X00000040;
}

inline int black_long_castle_flag(
        position_t * pos)
{
    return pos->bb[8] & 0X00000020;
}

inline int black_short_castle_flag(
        position_t * pos)
{
    return pos->bb[8] & 0X00000010;
}
