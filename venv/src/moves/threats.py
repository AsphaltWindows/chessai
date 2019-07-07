import chess_consts as cc


def is_threatened(byside, square, game):
    f = square[0]
    r = square[1]
    # King threats
    ktsquares = [(f, r + 1), (f, r - 1), (f + 1, r), (f + 1, r + 1), (f + 1, r - 1), (f - 1, r), (f - 1, r + 1),
                 (f - 1, r - 1)]
    ktsquares = [x for x in ktsquares if 0 < sq[0] < 9 and 0 < sq[1] < 9]

    for sq in ktsquares:
        piecenum = game.board[sq[0]][sq[1]]
        if piecenum is not None and game.pieces[piecenum][0] == byside and game.pieces[piecenum][1] == cc.King:
            return True

    # Knight threats
    ntsquares = [(f + 2, r + 1), (f + 2, r - 1), (f + 1, r + 2), (f + 1, r - 2), (f - 1, r + 2), (f - 1, r - 2),
                 (f - 2, r + 1), (f - 2, r - 1)]
    ntsquares = [x for x in ntsquares if 0 < sq[0] < 9 and 0 < sq[1] < 9]

    for sq in ntsquares:
        piecenum = game.board[sq[0]][sq[1]]
        if piecenum is not None and game.pieces[piecenum][0] == byside and game.pieces[piecenum][1] == cc.Knight:
            return True

    # Pawn threats
    ptsquares = []
    if byside == cc.White:
        ptsquares += [(f - 1, r + 1), (f + 1, r + 1)]
    else:
        ptsquares += [(f - 1, r - 1), (f + 1, r - 1)]

    ptsquares = [x for x in ptsquares if 0 < sq[0] < 9 and 0 < sq[1] < 9]

    for sq in ptsquares:
        piecenum = game.board[sq[0]][sq[1]]
        if piecenum is not None and game.pieces[piecenum][0] == byside and game.pieces[piecenum][1] == cc.Pawn:
            return True

    # Diagonal threats
    # top right
    f2 = f + 1
    r2 = r + 1

    while 0 < f2 < 9 and 0 < r2 < 9:
        piecenum = game.board[f2][r2]
        if piecenum is None:
            f2 += 1
            r2 += 1
        elif (game.pieces[piecenum][1] != cc.Bishop and game.pieces[piecenum][1] != cc.Queen) or game.pieces[piecenum][0] != byside:
            break
        else:
            return True
    # bottom right
    f2 = f + 1
    r2 = r - 1

    while 0 < f2 < 9 and 0 < r2 < 9:
        piecenum = game.board[f2][r2]
        if piecenum is None:
            f2 += 1
            r2 -= 1
        elif (game.pieces[piecenum][1] != cc.Bishop and game.pieces[piecenum][1] != cc.Queen) or game.pieces[piecenum][0] != byside:
            break
        else:
            return True

    # top left
    f2 = f - 1
    r2 = r + 1

    while 0 < f2 < 9 and 0 < r2 < 9:
        piecenum = game.board[f2][r2]
        if piecenum is None:
            f2 += 1
            r2 -= 1
        elif (game.pieces[piecenum][1] != cc.Bishop and game.pieces[piecenum][1] != cc.Queen) or game.pieces[piecenum][0] != byside:
            break
        else:
            return True

    # bottom left
    f2 = f - 1
    r2 = r + 1

    while 0 < f2 < 9 and 0 < r2 < 9:
        piecenum = game.board[f2][r2]
        if piecenum is None:
            f2 += 1
            r2 -= 1
        elif (game.pieces[piecenum][1] != cc.Bishop and game.pieces[piecenum][1] != cc.Queen) or game.pieces[piecenum][0] != byside:
            break
        else:
            return True

    # line threats
    # right
    f2 = f + 1

    while 0 < f2 < 9:
        piecenum = game.board[f2][r]
        if piecenum is None:
            f2 += 1
        elif (game.pieces[piecenum][1] != cc.Rook and game.pieces[piecenum][1] != cc.Queen) or game.pieces[piecenum][0] != byside:
            break
        else:
            return True

    # left
    f2 = f - 1

    while 0 < f2 < 9:
        piecenum = game.board[f2][r]
        if piecenum is None:
            f2 -= 1
        elif (game.pieces[piecenum][1] != cc.Rook and game.pieces[piecenum][1] != cc.Queen) or game.pieces[piecenum][0] != byside:
            break
        else:
            return True

    # top
    r2 = r + 1

    while 0 < r2 < 9:
        piecenum = game.board[f][r2]
        if piecenum is None:
            r2 += 1
        elif (game.pieces[piecenum][1] != cc.Rook and game.pieces[piecenum][1] != cc.Queen) or game.pieces[piecenum][0] != byside:
            break
        else:
            return True

    # bottom
    r2 = r - 1

    while 0 < r2 < 9:
        piecenum = game.board[f][r2]
        if piecenum is None:
            f2 -= 1
        elif (game.pieces[piecenum][1] != cc.Rook and game.pieces[piecenum][1] != cc.Queen) or game.pieces[piecenum][0] != byside:
            break
        else:
            return True

    return False


def is_valid(sidemoved, game):
    if sidemoved == cc.White:
        piecenum = 4
        threatened_by = cc.Black
    else:
        piecenum = 20
        threatened_by = cc.White
    return is_threatened(threatened_by, game.positions[piecenum], game)
