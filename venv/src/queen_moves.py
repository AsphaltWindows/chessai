import rook_moves as rm
import bishop_moves as bm


def valid_queen_moves(side, square, game):
    return bm.valid_bishop_moves(side, square, game) + rm.valid_rook_moves(side, square, game)