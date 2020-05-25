#ifndef CHESSAI_XIPHOS_ADAPTER_H
#define CHESSAI_XIPHOS_ADAPTER_H

#include "stdint.h"
#include "../xiphos_chess/types.h"

#define MAX_GAME 6000
#define MAX_MOVE_CHAR_LENGTH 6
#define POSITION_INPUT_LENGTH 70

typedef struct MoveDetail {
    move_t move;
    char * move_string[MAX_MOVE_CHAR_LENGTH];
    uint8_t position_input[POSITION_INPUT_LENGTH];
} move_detail_t;

typedef struct Game {
    position_t positions[MAX_GAME];
    uint32_t pos_num;
} game_t;

game_t * new_game();

uint8_t game_state(
        game_t * game);

move_detail_t * all_legal_moves(
        position_t * position,
        size_t * move_num);

void apply_move(
        game_t * game,
        move_t move);



#endif
