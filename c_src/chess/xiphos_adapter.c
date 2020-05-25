#include "xiphos_adapter.h"

game_t * new_game();

uint8_t game_state(
        game_t * game);

move_detail_t * all_legal_moves(
        position_t * position,
        size_t * move_num);

void apply_move(
        game_t * game,
        move_t move);


