#ifndef MOVE_H
#define MOVE_H

#include <stdint.h>

/** Right shift numbers**/
#define RS_PIECE_TO_MOVE 28
#define RS_PIECE_PLACED 24
#define RS_FILE_FROM 21
#define RS_RANK_FROM 18
#define RS_FILE_FROM 15
#define RS_RANK_FROM 12
#define RS_EN_PASSANT 9
#define RS_CASTLE 7

/**
 * 4 bits Piece Type PIECE_TO_MOVE being moved (NONE if castle)
 * 4 bits Piece Type PIECE_PLACED at target location after move (same as Piece Type being moved unless promotion, NONE if castle)
 * 3 bits File Type FILE_FROM
 * 3 bits Rank Type RANK_FROM
 * 3 bits File Type FILE_TO
 * 3 bits Rank Type RANK_TO
 * 3 bits EnPassant Type EN_PASSANT
 * 2 bits Castle Type CASTLE
 * 3 bits 000 empty 3 bits
 * **/

int is_pawn_move(
        move_t);

int is_capture(
        move_t);

#endif //MOVE_H
