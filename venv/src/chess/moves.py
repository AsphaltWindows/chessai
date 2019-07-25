from chess import chess_consts as cc
from chess import threats as t
from chess import bishop_moves as bm
from chess import knight_moves as nm
from chess import king_moves as km
from chess import pawn_moves as pm
from chess import rook_moves as rm
from chess import queen_moves as qm
from chess import castle_moves as cm


def all_legal_moves(game, pos_fn=lambda x: None):
    side = game.to_move
    legal_moves = []
    pos_res = []

    castle_moves = cm.valid_castle_moves(side, game)

    for move in castle_moves:
        game.apply_move(move)
        if t.is_legal(side, game):
            legal_moves.append(move)
            pos_res.append(pos_fn(game))
        game.unapply_move()

    for file in range(cc.A, cc.H + 1):
        for rank in range(1, 9):
            piecenum = game.board[rank][file][0]

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
                                pos_res.append(pos_fn(game))
                            game.unapply_move()
                    elif piecetype == cc.Knight:
                        valid_moves = nm.valid_knight_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                                pos_res.append(pos_fn(game))
                            game.unapply_move()
                    elif piecetype == cc.Bishop:
                        valid_moves = bm.valid_bishop_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                                pos_res.append(pos_fn(game))
                            game.unapply_move()
                    elif piecetype == cc.Rook:
                        valid_moves = rm.valid_rook_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                                pos_res.append(pos_fn(game))
                            game.unapply_move()
                    elif piecetype == cc.Queen:
                        valid_moves = qm.valid_queen_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                                pos_res.append(pos_fn(game))
                            game.unapply_move()
                    elif piecetype == cc.King:
                        valid_moves = km.valid_king_moves(side, (file, rank), game)
                        for move in valid_moves:
                            game.apply_move(move)
                            if t.is_legal(side, game):
                                legal_moves.append(move)
                                pos_res.append(pos_fn(game))
                            game.unapply_move()
    return legal_moves, pos_res
