/*
  Xiphos, a UCI chess engine
  Copyright (C) 2018, 2019 Milos Tatarevic

  Xiphos is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  Xiphos is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "bitboard.h"
#include "hash.h"
#include "position.h"

void set_pins_and_checks(position_t *pos)
{
  int k_sq, side, in_check;
  uint64_t occ_o, bq, rq, b_att, r_att;

  k_sq = pos->k_sq[pos->side];
  if (k_sq == NO_SQ) return;

  side = pos->side;
  pins_and_attacks_to(pos, pos->k_sq[side], side ^ 1, side,
                      &pos->pinned[side], &pos->pinners[side ^ 1],
                      &b_att, &r_att);

  occ_o = pos->occ[side ^ 1];
  bq = (pos->piece_occ[BISHOP] | pos->piece_occ[QUEEN]) & occ_o;
  rq = (pos->piece_occ[ROOK] | pos->piece_occ[QUEEN]) & occ_o;
  in_check = (b_att & bq) || (r_att & rq) ? 1 : 0;

  if (!in_check)
  {
    if (_b_piece_area[_f_piece(pos, PAWN)][k_sq] & pos->piece_occ[PAWN] & occ_o)
      in_check = 1;
    else if (_b_piece_area[KNIGHT][k_sq] & pos->piece_occ[KNIGHT] & occ_o)
      in_check = 1;
    else if (_b_piece_area[KING][k_sq] & _b(pos->k_sq[pos->side ^ 1]))
      in_check = 1;
  }
  pos->in_check = in_check;
}

int attacked(position_t *pos, int sq)
{
  uint64_t acc, full_occ, occ_o;

  occ_o = pos->occ[pos->side ^ 1];

  if (_b_piece_area[_f_piece(pos, PAWN)][sq] & pos->piece_occ[PAWN] & occ_o)
    return 1;
  if (_b_piece_area[KNIGHT][sq] & pos->piece_occ[KNIGHT] & occ_o)
    return 1;
  if (_b_piece_area[KING][sq] & _b(pos->k_sq[pos->side ^ 1]))
    return 1;

  full_occ = pos->occ[pos->side] | occ_o;

  acc = (pos->piece_occ[BISHOP] | pos->piece_occ[QUEEN]) & occ_o;
  if (bishop_attack(full_occ, sq) & acc)
    return 1;

  acc = (pos->piece_occ[ROOK] | pos->piece_occ[QUEEN]) & occ_o;
  if (rook_attack(full_occ, sq) & acc)
    return 1;

  return 0;
}

int attacked_after_move(position_t *pos, int sq, move_t move)
{
  int m_from, m_to, w_piece;
  uint64_t occ, occ_f, occ_o, occ_x, fb, tb;

  m_from = _m_from(move);
  m_to = _m_to(move);
  w_piece = _to_white(pos->board[m_from]);

  fb = _b(m_from);
  tb = _b(m_to);

  occ_o = pos->occ[pos->side ^ 1];
  if (pos->board[m_to] != EMPTY)
    occ_o ^= tb;
  else if (w_piece == PAWN && pos->ep_sq == m_to)
    occ_o ^= _b(m_to ^ 8);

  if (_b_piece_area[_f_piece(pos, PAWN)][sq] & pos->piece_occ[PAWN] & occ_o)
    return 1;
  if (_b_piece_area[KNIGHT][sq] & pos->piece_occ[KNIGHT] & occ_o)
    return 1;
  if (_b_piece_area[KING][sq] & _b(pos->k_sq[pos->side ^ 1]))
    return 1;

  occ_f = (pos->occ[pos->side] ^ fb) | tb;
  occ = occ_f | occ_o;

  occ_x = (pos->piece_occ[BISHOP] | pos->piece_occ[QUEEN]) & occ_o;
  if (bishop_attack(occ, sq) & occ_x)
    return 1;

  occ_x = (pos->piece_occ[ROOK] | pos->piece_occ[QUEEN]) & occ_o;
  if (rook_attack(occ, sq) & occ_x)
    return 1;

  return 0;
}

int is_pseudo_legal(position_t *pos, move_t move)
{
  int piece, t_piece, m_from, m_to, m_diff;
  uint64_t p_area, occ;

  m_from = _m_from(move);
  piece = pos->board[m_from];
  if (piece == EMPTY || _side(piece) != pos->side)
    return 0;

  m_to = _m_to(move);
  t_piece = pos->board[m_to];
  if (t_piece != EMPTY && _side(piece) == _side(t_piece))
    return 0;

  piece = _to_white(piece);
  m_diff = m_to - m_from;

  if (piece == PAWN)
  {
    if (!_m_promoted_to(move) && (_rank(m_to) == RANK_8 || _rank(m_to) == RANK_1))
      return 0;

    occ = _occ(pos);
    p_area = pushed_pawns(_b(m_from), ~occ, pos->side) & _b(m_to);
    if (p_area)
      return 1;

    p_area = pawn_attacks(_b(m_from), pos->side) & _b(m_to);
    if (p_area)
    {
      if (t_piece != EMPTY)
        return 1;
      else if (m_to == pos->ep_sq)
        return 1;
    }
    return 0;
  }

  if (piece != KING)
  {
    if (!(_b_piece_area[piece][m_from] & _b(m_to)))
      return 0;

    if (piece == KNIGHT)
      return 1;

    return (_b_line[m_from][m_to] & _occ(pos)) == 0;
  }

  // castling
  if (m_diff == 2 || m_diff == -2)
  {
    if (!pos->c_flag || pos->board[m_to] != EMPTY ||
         pos->board[(m_from + m_to) >> 1] != EMPTY)
      return 0;

    if (m_diff > 0)
      return pos->c_flag & ((pos->side == WHITE) ? C_FLAG_WR : C_FLAG_BR);
    else
      return pos->board[m_to - 1] == EMPTY &&
            (pos->c_flag & ((pos->side == WHITE) ? C_FLAG_WL : C_FLAG_BL));
  }

  return !!(_b_piece_area[KING][m_from] & _b(m_to));
}

int legal_move(position_t *pos, move_t move)
{
  int m_from, m_to, w_piece, m_diff, k_sq;
  uint64_t pinned;

  k_sq = pos->k_sq[pos->side];
  if (k_sq == NO_SQ) return 0;

  m_from = _m_from(move);
  m_to = _m_to(move);

  pinned = pos->pinned[pos->side];
  w_piece = _to_white(pos->board[m_from]);
  if (pinned == 0 && w_piece != KING && !pos->in_check &&
     (pos->ep_sq != m_to || w_piece != PAWN))
    return 1;

  if (w_piece == KING)
  {
    m_diff = m_to - m_from;
    if ((m_diff == 2 || m_diff == -2) &&
        (pos->in_check || attacked(pos, (_m_from(move) + _m_to(move)) >> 1)))
      return 0;
    return !attacked_after_move(pos, m_to, move);
  }

  if (pos->in_check || (pos->ep_sq == m_to && w_piece == PAWN))
    return !attacked_after_move(pos, k_sq, move);

  if ((pinned & _b(m_from)) == 0)
    return 1;

  return (_b_line[k_sq][m_to] & _b(m_from)) || (_b_line[k_sq][m_from] & _b(m_to));
}

int insufficient_material(position_t *pos)
{
  uint64_t occ;

  occ = _occ(pos);
  return _popcnt(occ) == 3 &&
         (occ & (pos->piece_occ[KNIGHT] | pos->piece_occ[BISHOP]));
}

