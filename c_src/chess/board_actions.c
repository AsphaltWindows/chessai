#include "board_actions.h"

#include "utils.h"

static uint32_t move_from_value(
        position_t *pos,
        move_t mov);

static uint32_t move_to_value(
        position_t *pos,
        move_t mov);

static uint32_t castle_value(
        position_t *pos,
        move_t mov);

static uint32_t flags_value(
        position_t *pos,
        move_t mov);

static void move_from_to_castle_index(
        position_t *pos,
        move_t mov,
        int *from_idx,
        int *to_idx,
        int *castle_idx);

void starting_position(
        position_t *to_change);

void do_move(
        position_t const *from,
        move_t move,
        position_t *to) {
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

    to->bb[bb_from_idx] = move_from_value(to, move);
    to->bb[bb_to_idx] = move_to_value(to, move);
    to->bb[bb_castle_idx] = castle_value(to, move);
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
        position_t *pos,
        move_t mov) {
    file_t f = file_to(mov);
    rank_t r = rank_to(mov);

    return pos->bb[r] & (~(15 << (f * 4)));
}

static uint32_t move_to_value(
        position_t *pos,
        move_t mov) {
    piece_t p = (move >> RS_PIECE_PLACED & 15);
    file_t f = file_to(mov);
    rank_t r = rank_to(mov);

    return (pos->bb[r] & (~(15 << (f * 4)))) | (p << (f * 4));
}

static uint32_t castle_value(
        position_t *pos,
        move_t mov) {
    uint32_t castling = (mov >> RS_CASTLE) & 7;
    uint32_t color = castling >> 2;
    uint32_t idx_shift = color + (color << 1);
    int idx = (1 << idx_shift) - 1;
    uint32_t long_castle_fill = ~((uint32_t)((castling & 1) - 1));
    uint32_t short_castle_fill = ~long_castle_fill;
    uint32_t black_fill = ~(color - 1);
    uint32_t white_fill = ~black_fill;

    uint32_t blank_mask = (long_castle_fill & LONG_CASTLE_BLANK) |
                          (short_castle_fill & SHORT_CASTLE_BLANK);
    uint32_t blanked = pos->bb[idx] & blank_mask;
    uint32_t fill_mask = (white_fill & long_castle_fill & WHITE_LONG_CASTLE_FILL) |
                         (white_fill & short_castle_fill & WHITE_SHORT_CASTLE_FILL) |
                         (black_fill & long_castle_fill & BLACK_LONG_CASTLE_FILL) |
                         (black_fill & short_castle_fill & BLACK_SHORT_CASTLE_FILL);
    return blanked & fill_mask;
}

static uint32_t flags_value(
        position_t *pos,
        move_t mov)
{
    uint32_t current_value = pos->bb[FLAGS_INDEX] ^ 1;
    uint32_t current_castle_flags = (current_value >> RS_CASTLE_FLAG) & 15;

    color_t side = move_side(mov);
    uint32_t castle_flags_shift = (!side) << 1;
    uint32_t if_none_flags = (~(3 << castle_flags_shift)) & 15;
    uint32_t if_nolong_flags = (~(2 << castle_flags_shift)) & 15;
    uint32_t if_noshort_flags = (~(1 << castle_flags_shift)) & 15;

    uint32_t s_from = square_from(mov);
    uint32_t s_to = square_to(mov);
    uint32_t rank = pos->bb[(1 << (side + (side << 1))) - 1];

    uint32_t king_move_flags = if_none_flags |
            (~ ((uint32_t) (!(is_castle(mov) || s_from == square(E, rank)) - 1)) & 15);
    uint32_t arook_move_flags = if_nolong_flags |
            (~ ((uint32_t) (!(s_from == square(A, rank) || s_to == square(A, rank)) - 1)) & 15);
    uint32_t hrook_move_flags = if_noshort_flags |
            (~ ((uint32_t) (!(s_from == square(H, rank) || s_to == square(H, rank)) - 1)) & 15);

    uint32_t castle_flags_mask = king_move_flags &
            arook_move_flags &
            hrook_move_flags;

    uint32_t trigger_enpassant = ((~ ((uint32_t) (is_pawn_double(mov) - 1))) & 15) & (8 | rank_from(mov));

    return


//    int castling_mask;
//    int enpassant_mask;
//    int w_l_castle;
//    int w_s_castle;
//    int b_l_castle;
//    int b_s_castle;
//
//    w_l_castle = ((to->bb[8] >> RS_WHITE_LONG_CASTLE) & 1) || is_castle(mov);
//    w_s_castle = ((to->bb[8] >> RS_WHITE_SHORT_CASTLE) & 1) || is_castle(mov);
//    b_l_castle = ((to->bb[8] >> RS_BLACK_LONG_CASTLE) & 1) || is_castle(mov);
//    w_s_castle = ((to->bb[8] >> RS_BLACK_SHORT_CASTLE) & 1) || is_castle(mov);

}

static int move_from_to_castle_index(
        position_t *pos,
        move_t mov,
        int *from_idx,
        int *to_idx,
        int *castle_idx);

{

int castle = is_castle(mov);

int castle_mask = castle | (castle << 1) | (castle << 2) | (castle << 3);

}


move_t *legal_moves(
        position_t *pos_ptr,
        size_t *outsize);

