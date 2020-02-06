#ifndef CHESS_MOVE_H
#define CHESS_MOVE_H

#include <stdint.h>
#include "types.h"

/** Right shift numbers**/
#define RS_PIECE_TO_MOVE 28
#define RS_PIECE_PLACED 24
#define RS_PIECE_REMOVED 20
#define RS_FILE_FROM 17
#define RS_RANK_FROM 14
#define RS_SQUARE_FROM 14
#define RS_FILE_TO 11
#define RS_RANK_TO 8
#define RS_SQUARE_TO 8
#define RS_EN_PASSANT 5
#define RS_CASTLE_COLOR 4
#define RS_CASTLE 3

/**
 * 4 bits Piece PIECE_TO_MOVE being moved (NONE if castle)
 * 4 bits Piece PIECE_PLACED at target location after move (same as Piece Type being moved unless promotion, NONE if castle)
 * 4 bits Piece PIECE_REMOVED at target location before the move (NONE if not a capture)
 * 3 bits File Type FILE_FROM
 * 3 bits Rank Type RANK_FROM
 * 3 bits File Type FILE_TO
 * 3 bits Rank Type RANK_TO
 * 3 bits EnPassant Type EN_PASSANT
 * 2 bits Castle Type CASTLE
 * 3 bits 000 empty 3 bits
 * **/

inline color_t move_side(
        move_t move);

/** 1 true 0 false **/
inline int is_pawn_move(
        move_t mov);

/** 1 true 0 false **/
inline int is_pawn_double(
        move_t mov);

/** 1 true 0 false **/
inline int is_capture(
        move_t mov);

/** 1 true 0 false **/
inline int is_castle(
        move_t mov);

inline rank_t rank_from(
        move_t mov);

inline file_t file_from(
        move_t mov);

inline square_t square_from(
        move_t mov);

inline rank_t rank_to(
        move_t mov);

inline file_t file_to(
        move_t mov);

inline square_t square_to(
        move_t mov);

#endif //MOVE_H
