#include "board_actions.h"

void starting_position(
        position_t * to_change);

void do_move(
        position_t const * from,
        move_t move,
        position_t * to)
{
    memcpy(&(to->bb[0]), &(from->bb[0]), sizeof(position_t));

    move >> RS_FILE_FROM
}

void undo_move(
        position_t const * from,
        move_t move,
        position_t * to);

move_t * legal_moves(
        position_t * pos_ptr,
        size_t * outsize);

