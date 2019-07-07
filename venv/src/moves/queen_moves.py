import rook_moves as rm
import bishop_moves as bm


def valid_queen_moves(side, square, game):
    return rm.valid_bishop_moves(side, square, game) + bm.valid_rook_moves(side, square, game)