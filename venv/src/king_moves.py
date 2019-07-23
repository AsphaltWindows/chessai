import chess_consts as cc
import chess_move as cm

kingMoves = {}
kingCaptures = {}

for file in range(cc.A, cc.H + 1):
    for rank in range(1, 9):
        fr = (file, rank)
        squares = []
        moves = []
        captures = []
        squares.append((file, rank - 1))
        squares.append((file, rank + 1))
        squares.append((file - 1, rank))
        squares.append((file - 1, rank + 1))
        squares.append((file - 1, rank - 1))
        squares.append((file + 1, rank))
        squares.append((file + 1, rank + 1))
        squares.append((file + 1, rank - 1))

        for sq in squares:
            if (sq[0] > 0) and (sq[0] < 9) and (sq[1] > 0) and (sq[1] < 9):
                moves.append(cm.ChessMove(fr, sq))
                captures.append(cm.ChessMove(fr, sq, True))

        kingMoves.update({fr: moves})
        kingCaptures.update({fr: captures})


def valid_king_moves(side, square, game):
    valid_moves = []
    for move in kingMoves[square]:
        if game.board[move.to[1]][move.to[0]][0] is None:
            valid_moves.append(move)

    for move in kingCaptures[square]:
        piecenum = game.board[move.to[1]][move.to[0]][0]
        if piecenum is not None and game.pieces[piecenum][0] != side and game.pieces[piecenum][1] != cc.King:
            valid_moves.append(move)

    return valid_moves
