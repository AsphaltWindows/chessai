from chess import bishop_moves as bm
from chess import pawn_moves as pm
from chess import knight_moves as nm
from chess import king_moves as km
from chess import rook_moves as rm
from chess import castle_moves as cm
from chess import chess_parse as cp
from chess import chess_consts as cc

string_to_move = {}

move_to_string = {}

black_string_to_move = {}
white_string_to_move = {}

for fr in pm.whitePawnMoves:
    for move in pm.whitePawnMoves[fr]:
        string = cp.square_to_string(fr) + '-' + cp.square_to_string(move.to)

        if move.promote is not None:
            string += ('=' + cp.piece_to_letter(move.promote))

        white_string_to_move.update({string: move})

for fr in pm.blackPawnMoves:
    for move in pm.blackPawnMoves[fr]:
        string = cp.square_to_string(fr) + '-' + cp.square_to_string(move.to)

        if move.promote is not None:
            string += ('=' + cp.piece_to_letter(move.promote))

        black_string_to_move.update({string: move})

for fr in pm.whitePawnDoubleMoves:
    for move in pm.whitePawnDoubleMoves[fr]:
        string = cp.square_to_string(fr) + '-' + cp.square_to_string(move.to)
        white_string_to_move.update({string: move})

for fr in pm.blackPawnMoves:
    for move in pm.blackPawnDoubleMoves[fr]:
        string = cp.square_to_string(fr) + '-' + cp.square_to_string(move.to)
        black_string_to_move.update({string: move})

for fr in pm.whitePawnCaptures:
    for move in pm.whitePawnCaptures[fr]:
        string = cp.square_to_string(fr) + 'x' + cp.square_to_string(move.to)

        if move.promote is not None:
            string += ('=' + cp.piece_to_letter(move.promote))

        white_string_to_move.update({string: move})

for fr in pm.blackPawnCaptures:
    for move in pm.blackPawnCaptures[fr]:
        string = cp.square_to_string(fr) + 'x' + cp.square_to_string(move.to)

        if move.promote is not None:
            string += ('=' + cp.piece_to_letter(move.promote))

        black_string_to_move.update({string: move})

for fr in pm.whitePawnEnPassant:
    for move in pm.whitePawnEnPassant[fr]:
        string = cp.square_to_string(fr) + 'x' + cp.square_to_string(move.to)
        white_string_to_move.update({string: move})

for fr in pm.blackPawnEnPassant:
    for move in pm.blackPawnEnPassant[fr]:
        string = cp.square_to_string(fr) + 'x' + cp.square_to_string(move.to)
        black_string_to_move.update({string: move})

for fr in nm.knightMoves:
    for move in nm.knightMoves[fr]:
        string = 'N' + cp.square_to_string(fr) + '-' + cp.square_to_string(move.to)
        white_string_to_move.update({string: move})
        black_string_to_move.update({string: move})

for fr in nm.knightCaptures:
    for move in nm.knightCaptures[fr]:
        string = 'N' + cp.square_to_string(fr) + 'x' + cp.square_to_string(move.to)
        white_string_to_move.update({string: move})
        black_string_to_move.update({string: move})

for fr in km.kingMoves:
    for move in km.kingMoves[fr]:
        string = 'K' + cp.square_to_string(fr) + '-' + cp.square_to_string(move.to)
        white_string_to_move.update({string: move})
        black_string_to_move.update({string: move})

for fr in km.kingCaptures:
    for move in km.kingMoves[fr]:
        string = 'K' + cp.square_to_string(fr) + 'x' + cp.square_to_string(move.to)
        white_string_to_move.update({string: move})
        black_string_to_move.update({string: move})

for fr in bm.bishopVectors:
    for vector in bm.bishopVectors[fr]:
        for move_pair in vector:
            bmString = 'B' + cp.square_to_string(fr) + '-' + cp.square_to_string(move_pair[0].to)
            qmString = 'Q' + cp.square_to_string(fr) + '-' + cp.square_to_string(move_pair[0].to)
            bcString = 'B' + cp.square_to_string(fr) + 'x' + cp.square_to_string(move_pair[1].to)
            qcString = 'Q' + cp.square_to_string(fr) + 'x' + cp.square_to_string(move_pair[1].to)
            white_string_to_move.update({bmString: move_pair[0]})
            black_string_to_move.update({bmString: move_pair[0]})
            white_string_to_move.update({qmString: move_pair[0]})
            black_string_to_move.update({qmString: move_pair[0]})
            white_string_to_move.update({bcString: move_pair[1]})
            black_string_to_move.update({bcString: move_pair[1]})
            white_string_to_move.update({qcString: move_pair[1]})
            black_string_to_move.update({qcString: move_pair[1]})

for fr in rm.rookVectors:
    for vector in rm.rookVectors[fr]:
        for move_pair in vector:
            rmString = 'R' + cp.square_to_string(fr) + '-' + cp.square_to_string(move_pair[0].to)
            qmString = 'Q' + cp.square_to_string(fr) + '-' + cp.square_to_string(move_pair[0].to)
            rcString = 'R' + cp.square_to_string(fr) + 'x' + cp.square_to_string(move_pair[1].to)
            qcString = 'Q' + cp.square_to_string(fr) + 'x' + cp.square_to_string(move_pair[1].to)
            white_string_to_move.update({rmString: move_pair[0]})
            black_string_to_move.update({rmString: move_pair[0]})
            white_string_to_move.update({qmString: move_pair[0]})
            black_string_to_move.update({qmString: move_pair[0]})
            white_string_to_move.update({rcString: move_pair[1]})
            black_string_to_move.update({rcString: move_pair[1]})
            white_string_to_move.update({qcString: move_pair[1]})
            black_string_to_move.update({qcString: move_pair[1]})

white_string_to_move.update({'O-O-O': cm.longWhiteCastle})
black_string_to_move.update({'O-O-O': cm.longBlackCastle})
white_string_to_move.update({'O-O': cm.shortWhiteCastle})
black_string_to_move.update({'O-O': cm.shortBlackCastle})

string_to_move = {
    cc.White: white_string_to_move,
    cc.Black: black_string_to_move
}


def parse_move(side, move_string):
    return string_to_move[side][move_string]


def move_to_string(mv, game):
    if mv.longCastleWhite or mv.longCastleBlack:
        return "O-O-O"
    elif mv.shortCastleWhite or mv.shortCastleBlack:
        return "O-O"
    else:
        piecenum = game.board[mv.fr[1]][mv.fr[0]][0]
        piecetype = game.pieces[piecenum][1]
        move_str = ""
        move_str += cp.piece_to_letter(piecetype)
        move_str += cp.square_to_string(mv.fr)
        if mv.capture:
            move_str += "x"
        else:
            move_str += "-"
        move_str += cp.square_to_string(mv.to)

        if mv.promote is not None:
            move_str += "="
            move_str += cp.piece_to_letter(mv.promote)

        return move_str
