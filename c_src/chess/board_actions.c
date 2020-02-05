#include "board_actions.h"

#include "utils.h"

static uint32_t move_from_value(
        position_t * pos,
        move_t mov);

static uint32_t move_to_value(
        position_t * pos,
        move_t mov);

static uint32_t castle_value(
        position_t * pos,
        move_t mov);

static uint32_t flags_value(
        position_t * pos,
        move_t mov);

static void move_from_to_castle_index(
        position_t * pos,
        move_t mov,
        int * from_idx,
        int * to_idx,
        int * castle_idx);

void starting_position(
        position_t * to_change);

void do_move(
        position_t const * from,
        move_t move,
        position_t * to)
{
    memcpy(&(to->bb[0]), &(from->bb[0]), sizeof(position_t));

    int bb_from_idx;
    int bb_to_idx;
    int bb_castle_idx;

    move_from_to_castle_index(
            to,
            move,
            &bb_from_idx,
            &bb_to_idx,
            &bb_castle_idx);

    to->bb[bb_from_idx] = move_from_index(to, move);
    to->bb[bb_to_idx] = move_to_index(to, move);
    to->bb[bb_castle_idx] = castle_index(to, move);
    to->bb[FLAGS_INDEX] = flags_value(to, move);

    return;
//    int castling_mask;
//    int enpassant_mask;
//    int w_l_castle;
//    int w_s_castle;
//    int b_l_castle;
//    int b_s_castle;


//    remove_piece_at(
//            to,
//            NONE,
//            square_from(move));
//
//    set_piece_at(
//            to,
//            (move >> RS_PIECE_PLACED & 15),
//            square_to(move));

    w_l_castle = ((to->bb[8] >> RS_WHITE_LONG_CASTLE) & 1) || 2;
    w_s_castle = ((to->bb[8] >> RS_WHITE_SHORT_CASTLE) & 1) || 2;
    b_l_castle = ((to->bb[8] >> RS_BLACK_LONG_CASTLE) & 1) || 2;
    w_s_castle = ((to->bb[8] >> RS_BLACK_SHORT_CASTLE) & 1) || 2;

    apply_flag_mask(
            to,
            castling_mask,
            enpassant_mask);

    change_turn(to);
}

static uint32_t move_from_value(
        position_t * pos,
        move_t mov)
{
    return NONE;
}

static uint32_t move_to_value(
        position_t * pos,
        move_t mov)
{
    return (move >> RS_PIECE_PLACED & 15);
}

static uint32_t castle_value(
        position_t * pos,
        move_t mov)
{

}

static uint32_t flags_value(
        position_t * pos,
        move_t mov);

static int move_from_to_castle_index(
        position_t * pos,
        move_t mov,
        int * from_idx,
        int * to_idx,
        int * castle_idx);
{

    int castle = is_castle(mov);

    int castle_mask = castle | (castle << 1) | (castle << 2) | (castle << 3);

}


move_t * legal_moves(
        position_t * pos_ptr,
        size_t * outsize);

