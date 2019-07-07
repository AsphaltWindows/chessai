import chess_consts as cc
import chess_move as cm
import threats as t

longWhiteCastle = cm.ChessMove(None, None, False, 0, None, True, False, False, False)
longBlackCastle = cm.ChessMove(None, None, False, 0, None, False, True, False, False)
shortWhiteCastle = cm.ChessMove(None, None, False, 0, None, False, False, True, False)
shortBlackCastle = cm.ChessMove(None, None, False, 0, None, False, False, False, True)

def valid_castle_moves(side, game):
    valid_moves = []

    if side == cc.White:
        if game.white_long_castle():
            valid = True
            for sq in [(cc.E, 1), (cc.D, 1), (cc.C, 1)]:
                valid = valid and t.is_threatened(cc.Black, sq, game)
            if valid:
                valid_moves.append(longWhiteCastle)

        if game.white_short_castle():
            valid = True
            for sq in [(cc.E, 1), (cc.F, 1), (cc.G, 1)]:
                valid = valid and  t.is_threatened(cc.Black, sq, game)
            if valid:
                valid_moves.append(shortWhiteCastle)
    else:
        if game.black_long_castle():
            valid = True
            for sq in [(cc.E, 8), (cc.D, 8), (cc.C, 8)]:
                valid = valid and t.is_threatened(cc.White, sq, game)
            if valid:
                valid_moves.append(longBlackCastle)

        if game.black_short_castle():
            valid = True
            for sq in [(cc.E, 8), (cc.F, 8), (cc.G, 8)]:
                valid = valid and t.is_threatened(cc.White, sq, game)
            if valid:
                valid_moves.append(shortBlackCastle)


