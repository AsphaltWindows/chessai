from chess import chess_consts as cc
from chess import chess_move as cm

bishopVectors = {}

for file in range(cc.A, cc.H + 1):
    for rank in range(1, 9):
        fr = (file, rank)
        f = file + 1
        r = rank + 1
        plusPlus = []
        while f < 9 and r < 9:
            plusPlus.append((cm.ChessMove(fr, (f, r)), cm.ChessMove(fr, (f, r), True)))
            f += 1
            r += 1
        f = file + 1
        r = rank - 1
        plusMinus = []
        while f < 9 and r > 0:
            plusMinus.append((cm.ChessMove(fr, (f, r)), cm.ChessMove(fr, (f, r), True)))
            f += 1
            r -= 1
        f = file - 1
        r = rank + 1
        minusPlus = []
        while f > 0 and r < 9:
            minusPlus.append((cm.ChessMove(fr, (f, r)), cm.ChessMove(fr, (f, r), True)))
            f -= 1
            r += 1
        f = file - 1
        r = rank - 1
        minusMinus = []
        while f > 0 and r > 0:
            minusMinus.append((cm.ChessMove(fr, (f, r)), cm.ChessMove(fr, (f, r), True)))
            f -= 1
            r -= 1
        bishopVectors.update({fr: [plusPlus, plusMinus, minusPlus, minusMinus]})


def valid_bishop_moves(side, square, game):
    valid_moves = []
    for vector in bishopVectors[square]:
        for movepair in vector:
            to = movepair[0].to
            piecenum = game.board[to[1]][to[0]][0]
            if piecenum is None:
                valid_moves.append(movepair[0])
            elif game.pieces[piecenum][0] != side and game.pieces[piecenum][1] != cc.King:
                valid_moves.append(movepair[1])
                break
            else:
                break

    return valid_moves
