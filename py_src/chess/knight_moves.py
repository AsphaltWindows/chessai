from chess import chess_consts as cc
from chess import chess_move as cm

knightMoves = {}
knightCaptures = {}

for file in range(cc.A, cc.H + 1):
    for rank in range(1, 9):
        fr = (file, rank)
        squares = []
        moves = []
        captures = []
        squares.append((file - 2, rank - 1))
        squares.append((file - 2, rank + 1))
        squares.append((file - 1, rank - 2))
        squares.append((file - 1, rank + 2))
        squares.append((file + 1, rank - 2))
        squares.append((file + 1, rank + 2))
        squares.append((file + 2, rank - 1))
        squares.append((file + 2, rank + 1))

        for sq in squares:
            if (sq[0] > 0) and (sq[0] < 9) and (sq[1] > 0) and (sq[1] < 9):
                moves.append(cm.ChessMove(fr, sq))
                captures.append(cm.ChessMove(fr, sq, True))

        knightMoves.update({fr: moves})
        knightCaptures.update({fr: captures})


def valid_knight_moves(side, square, game):
    valid_moves = []
    for move in knightMoves[square]:
        if game.board[move.to[1]][move.to[0]][0] is None:
            valid_moves.append(move)

    for move in knightCaptures[square]:
        piecenum = game.board[move.to[1]][move.to[0]][0]
        if piecenum is not None and game.pieces[piecenum][0] != side and game.pieces[piecenum][1] != cc.King:
            valid_moves.append(move)

    return valid_moves
