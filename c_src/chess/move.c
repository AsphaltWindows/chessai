#include "move.h"

inline int is_pawn_move(
        move_t mov)
{
    return mov >> (RS_PIECE_TO_MOVE + 1) == PAWN;
}

inline int is_pawn_double(
        move_t mov)
{
    return is_pawn_move(mov) && ((rank_to(mov) == R4 && rank_from(mov) == R2) || (rank_to(mov) == R5 && rank_from(mov) == R7));
}

inline int is_capture(
        move_t mov)
{
    return mov >> (RS_PIECE_REMOVED + 1) != 7;
}

inline int is_castle(
        move_t mov)
{
    return mov >> (RS_CASTLE) != 0;
}

inline file_t file_from(
        move_t mov)
{
    return (file_t) ((mov >> RS_FILE_FROM) & 7);
}

inline rank_t rank_from(
        move_t mov)
{
    return (rank_t)((mov >> RS_RANK_FROM) & 7);
}

inline square_t square_from(
        move_t mov)
{
    return (square_t)((mov >> RS_SQUARE_FROM) & 63);
}

inline file_t file_to(
        move_t mov)
{
    return (file_t)((mov >> RS_FILE_TO) & 7);
}

inline rank_t rank_to(
        move_t mov)
{
    return (rank_t)((mov >> RS_RANK_TO) & 7);
}

inline square_t square_to(
        move_t mov)
{
    return (square_t)((mov >> RS_SQUARE_TO) & 7);
}

