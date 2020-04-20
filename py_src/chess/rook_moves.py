from chess import chess_consts as cc
from chess import chess_move as cm

rookVectors = {}

for file in range(cc.A, cc.H + 1):
    for rank in range(1, 9):
        fr = (file, rank)
        f = file + 1
        r = rank
        plusSame = []
        while f < 9:
            plusSame.append((cm.ChessMove(fr, (f, r)), cm.ChessMove(fr, (f, r), True)))
            f += 1
        f = file
        r = rank + 1
        samePlus = []
        while r < 9:
            samePlus.append((cm.ChessMove(fr, (f, r)), cm.ChessMove(fr, (f, r), True)))
            r += 1
        f = file - 1
        r = rank
        minusSame = []
        while f > 0:
            minusSame.append((cm.ChessMove(fr, (f, r)), cm.ChessMove(fr, (f, r), True)))
            f -= 1
        f = file
        r = rank - 1
        sameMinus = []
        while r > 0:
            sameMinus.append((cm.ChessMove(fr, (f, r)), cm.ChessMove(fr, (f, r), True)))
            r -= 1
        rookVectors.update({fr: [plusSame, samePlus, minusSame, sameMinus]})


def valid_rook_moves(side, square, game):
    valid_moves = []
    for vector in rookVectors[square]:
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
