from chess import threats as t
from chess import chess_consts as cc
from chess import moves as m


def game_state(game):
    to_move = game.to_move
    if to_move == cc.White:
        opponent = cc.Black
        king_pos = game.positions[4][0]
    else:
        opponent = cc.White
        king_pos = game.positions[20][0]

    moves = m.all_legal_moves(game)[0]
    phash = game.position_hash()

    if len(moves) == 0 and t.is_threatened(opponent, king_pos, game):
        return opponent
    elif len(moves) == 0:
        return cc.Draw
    elif phash in game.repetitions and game.repetitions[phash] == 3:
        return cc.Draw
    elif game.fiftyMoveRuleCount[0] == 100:
        return cc.Draw
    else:
        knightCount = 0
        lightBishopCount = 0
        darkBishopCount = 0
        for f in range(cc.A, cc.H + 1):
            for r in range(1, 9):
                piecenum = game.board[r][f][0]

                if piecenum is not None:
                    piecetype = game.pieces[piecenum][1]
                    if piecetype == cc.Queen or piecetype == cc.Rook or piecetype == cc.Pawn:
                        return cc.InProgress
                    elif piecetype == cc.Knight:
                        knightCount += 1
                    elif piecetype == cc.Bishop:
                        if (f + r) % 2 == 0:
                            darkBishopCount += 1
                        else:
                            lightBishopCount += 1
        if knightCount + lightBishopCount + darkBishopCount == 1:
            return cc.Draw
        elif lightBishopCount == 0 and knightCount == 0:
            return cc.Draw
        elif darkBishopCount == 0 and knightCount == 0:
            return cc.Draw
        else:
            return cc.InProgress
