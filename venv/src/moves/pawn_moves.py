import chess_consts as cc
import chess_moves as cm

whitePawnMoves = {}
whitePawnDoubleMoves = {}
whitePawnCaptures = {}
whitePawnEnPassant = {}
blackPawnMoves = {}
blackPawnDoubleMoves = {}
blackPawnCaptures = {}
blackPawnEnPassant = {}

for file in range(cc.A, cc.H + 1):
    for rank in range(2, 8):
        fr = (file, rank)
        moves = []
        double = []
        captures = []
        enPassant = []
        if rank < 7:
            moves.append(cm.ChessMove(fr, (file, rank + 1)))
            if file > cc.A:
                captures.append(cm.ChessMove(fr, (file - 1, rank + 1), True))
            if file < cc.H:
                captures.append(cm.ChessMove(fr, (file + 1, rank + 1), True))
        if rank == 2:
            double.append(cm.ChessMove(fr, (file, rank + 2)))
        elif rank == 5:
            if file > cc.A:
                captures.append(cm.ChessMove(fr, (file - 1, rank + 1), True, file - 1))
            if file < cc.H:
                captures.append(cm.ChessMove(fr, (file + 1, rank + 1), True, file + 1))
        if rank == 7:
            moves.append(cm.ChessMove(fr, (file, rank + 1), False, 0, cc.Knight))
            moves.append(cm.ChessMove(fr, (file, rank + 1), False, 0, cc.Bishop))
            moves.append(cm.ChessMove(fr, (file, rank + 1), False, 0, cc.Rook))
            moves.append(cm.ChessMove(fr, (file, rank + 1), False, 0, cc.Queen))
            if file > cc.A:
                captures.append(cm.ChessMove(fr, (file - 1, rank + 1), True, 0, cc.Knight))
                captures.append(cm.ChessMove(fr, (file - 1, rank + 1), True, 0, cc.Bishop))
                captures.append(cm.ChessMove(fr, (file - 1, rank + 1), True, 0, cc.Rook))
                captures.append(cm.ChessMove(fr, (file - 1, rank + 1), True, 0, cc.Queen))
            if file < cc.H:
                captures.append(cm.ChessMove(fr, (file + 1, rank + 1), True, 0, cc.Knight))
                captures.append(cm.ChessMove(fr, (file + 1, rank + 1), True, 0, cc.Bishop))
                captures.append(cm.ChessMove(fr, (file + 1, rank + 1), True, 0, cc.Rook))
                captures.append(cm.ChessMove(fr, (file + 1, rank + 1), True, 0, cc.Queen))
        whitePawnMoves.update({fr: moves})
        whitePawnCaptures.update({fr: captures})
        whitePawnDoubleMoves.update({fr: double})
        whitePawnEnPassant.update({fr: enPassant})

for file in range(cc.A, cc.H + 1):
    for rank in range(2, 8):
        fr = (file, rank)
        moves = []
        double = []
        captures = []
        enPassant = []
        if rank > 2:
            moves.append(cm.ChessMove(fr, (file, rank - 1)))
            if file > cc.A:
                captures.append(cm.ChessMove(fr, (file - 1, rank - 1), True))
            if file < cc.H:
                captures.append(cm.ChessMove(fr, (file + 1, rank - 1), True))
        if rank == 7:
            double.append(cm.ChessMove(fr, (file, rank - 2)))
        elif rank == 4:
            if file > cc.A:
                enPassant.append(cm.ChessMove(fr, (file - 1, rank - 1), True, file - 1))
            if file < cc.H:
                enPassant.append(cm.ChessMove(fr, (file + 1, rank - 1), True, file + 1))
        if rank == 2:
            moves.append(cm.ChessMove(fr, (file, rank - 1), False, 0, cc.Knight))
            moves.append(cm.ChessMove(fr, (file, rank - 1), False, 0, cc.Bishop))
            moves.append(cm.ChessMove(fr, (file, rank - 1), False, 0, cc.Rook))
            moves.append(cm.ChessMove(fr, (file, rank - 1), False, 0, cc.Queen))
            if file > cc.A:
                captures.append(cm.ChessMove(fr, (file - 1, rank - 1), True, 0, cc.Knight))
                captures.append(cm.ChessMove(fr, (file - 1, rank - 1), True, 0, cc.Bishop))
                captures.append(cm.ChessMove(fr, (file - 1, rank - 1), True, 0, cc.Rook))
                captures.append(cm.ChessMove(fr, (file - 1, rank - 1), True, 0, cc.Queen))
            if file < cc.H:
                captures.append(cm.ChessMove(fr, (file + 1, rank - 1), True, 0, cc.Knight))
                captures.append(cm.ChessMove(fr, (file + 1, rank - 1), True, 0, cc.Bishop))
                captures.append(cm.ChessMove(fr, (file + 1, rank - 1), True, 0, cc.Rook))
                captures.append(cm.ChessMove(fr, (file + 1, rank - 1), True, 0, cc.Queen))
        blackPawnMoves.update({fr: moves})
        blackPawnCaptures.update({fr: captures})
        blackPawnDoubleMoves.update({fr: double})
        blackPawnEnPassant.update({fr: enPassant})

pawnMoves = {
    cc.White: whitePawnMoves,
    cc.Black: blackPawnMoves
}

pawnCaptures = {
    cc.White: whitePawnCaptures,
    cc.Black: blackPawnCaptures
}

pawnDoubleMoves = {
    cc.White: whitePawnDoubleMoves,
    cc.Black: blackPawnDoubleMoves
}

pawnEnPassant = {
    cc.White: whitePawnEnPassant,
    cc.Black: blackPawnEnPassant
}


def valid_pawn_moves(side, square, game):
    valid_moves = []

    for move in pawnMoves[side][square]:
        if game.board[move.to[0]][move.to[1]][0] is None:
            valid_moves.append(move)

    for move in pawnCaptures[side][square]:
        piecenum = game.board[move.to[0]][move.to[1]][0]
        if piecenum is not None and game.pieces[piecenum][0] != side and game.pieces[piecenum][1] != cc.King:
            valid_moves.append(move)

    for move in pawnDoubleMoves[side][square]:
        if game.board[move.to[0]][move.to[1]][0] is not None and game.board[move.to[0]][(move.fr[1] + move.to[1])/2] is not None:
            valid_moves.append(move)

    for move in pawnEnPassant[side][square]:
        if game.enPassant[0] == move.enPassant:
            valid_moves.append(move)

    return valid_moves
