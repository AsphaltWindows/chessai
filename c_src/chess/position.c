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

    pos->bb[r] = (pos->bb[r] & (~ (15 << (f * 4)))) | (p << (f * 4));
    return;
}

inline void remove_piece_at(
        position_t * pos,
        piece_t p,
        square_t s)
{
    file_t f = square_file(s);
    rank_t r = square_rank(s);

    pos->bb[r] = pos->bb[r] & (~ (15 << (f * 4)));
    return;
}

//inline void apply_flag_mask(
//        position_t * pos,
//        int disable_castle,
//        int en_passant)
//{
//    pos->bb[FLAGS_INDEX] &= (0xFFFFF00F | (disable_castle << RS_CASTLE_FLAG) | (en_passant << RS_EN_PASSANT_FLAG));
//}

inline void disable_white_long_castle(
        position_t * pos)
{
    pos->bb[FLAGS_INDEX] &= 0xFFFFFF7F;
}

inline void disable_white_short_castle(
        position_t * pos)
{
    pos->bb[FLAGS_INDEX] &= 0xFFFFFFBF;
}

inline void disable_white_all_castle(
        position_t *pos)
{
    pos->bb[FLAGS_INDEX] &= 0xFFFFFF3F;
}

inline void disable_black_long_castle(
        position_t * pos)
{
    pos->bb[FLAGS_INDEX] &= 0xFFFFFFDF;
}

inline void disable_black_short_castle(
        position_t * pos)
{
    pos->bb[FLAGS_INDEX] &= 0xFFFFFFEF;
}

inline void disable_black_all_castle(
        position_t *pos)
{
    pos->bb[FLAGS_INDEX] &= 0xFFFFFFCF;
}

inline void change_turn(
        position_t * pos)
{
    pos->bb[FLAGS_INDEX] ^= 1;
    return;
}

inline void set_enpassant(
        position_t * pos,
        file_t f)
{
    pos->bb[FLAGS_INDEX] &= (0xFFFFF8FF | (f << 8));
}

inline void reset_enpassant(
        position_t * pos)
{
    pos->bb[FLAGS_INDEX] &= 0xFFFFF0FF;
}

inline int white_long_castle_flag(
        position_t * pos)
{
    return pos->bb[FLAGS_INDEX] & 0x00000080;
}

inline int white_short_castle_flag(
        position_t * pos)
{
    return pos->bb[FLAGS_INDEX] & 0x00000040;
}

inline int black_long_castle_flag(
        position_t * pos)
{
    return pos->bb[FLAGS_INDEX] & 0x00000020;
}

inline int black_short_castle_flag(
        position_t * pos)
{
    return pos->bb[FLAGS_INDEX] & 0x00000010;
}
