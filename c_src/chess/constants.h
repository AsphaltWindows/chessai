#ifndef CHESS_CONSTANTS_H
#define CHESS_CONSTANTS_H

#include "types.h"

#define WHITE_WIN 0
#define BLACK_WIN 1
#define DRAW 2
#define INCOMPLETE 3

extern move_t * piece_moves[PIECE_T_NUM][SQUARE_NUM];

extern move_t * piece_captures[PIECE_T_NUM][SQUARE_NUM];

#endif //CHESS_CONSTANTS_H
