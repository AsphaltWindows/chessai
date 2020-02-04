#include "move.h"

inline int is_pawn_move(
        move_t mov)
{
    return (mov >> (RS_PIECE_TO_MOVE + 1)) & PAWN;
}

inline int is_capture(
        move_t mov)
{
    return (mov >> (RS_PIECE_REMOVED + 1)) & 7;
}

inline int file_from(
        move_t mov)
{
    return (mov >> RS_FILE_FROM) & 7;
}

inline int rank_from(
        move_t mov)
{
    return (mov >> RS_RANK_FROM) & 7;
}

inline int file_to(
        move_t mov)
{
    return (mov >> RS_FILE_TO) & 7;
}

inline int rank_to(
        move_t mov)
{
    return (mov >> RS_RANK_TO) & 7;
}
