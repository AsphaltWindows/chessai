import chess_consts as cc
import threats as t
import bishop_moves as bm
import knight_moves as nm
import king_moves as km
import pawn_moves as pm
import rook_moves as rm
import queen_moves as qm
import castle_moves as cm


def all_legal_moves(game):
    side = game.to_move
    legal_moves = []

    castle_moves = cm.valid_castle_moves(side, game)

    for move in castle_moves:
        game.apply_move(move)
        if t.is_legal(side, game):
            legal_moves.append(move)
        game.unapply_move(move)

    for file in range(cc.A, cc.H + 1):
        for rank in range(1, 9):
            piecenum = game.board[file][rank]

            if piecenum is not None:
                pieceside = game.pieces[piecenum][0]
                piecetype = game.pieces[piecenum][1]

                if pieceside == side:

                    if piecetype == cc.Pawn:
                        valid_moves = pm.valid_pawn_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                            game.unapply_move(move)
                    elif piecetype == cc.Knight:
                        valid_moves = nm.valid_knight_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                            game.unapply_move(move)
                    elif piecetype == cc.Bishop:
                        valid_moves = bm.valid_bishop_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                            game.unapply_move(move)
                    elif piecetype == cc.Rook:
                        valid_moves = rm.valid_rook_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                            game.unapply_move(move)
                    elif piecetype == cc.Queen:
                        valid_moves = qm.valid_queen_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                            game.unapply_move(move)
                    elif piecetype == cc.King:
                        valid_moves = km.valid_king_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                            game.unapply_move(move)

    return legal_moves
