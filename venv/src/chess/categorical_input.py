import chess.chess_consts as cc


def game_as_input(game):
    input = []

    for r in range(1, 9):
        for f in range(cc.A, cc.H + 1):
            piecenum = game.board[r][f][0]

            if piecenum is None:
                input.append(0)
            else:
                input.append(piece_as_value(game.pieces[piecenum]))

    input.append(game.to_move)
    input.append(game.enPassant[0])

    if game.white_long_castle():
        input.append(1)
    else:
        input.append(0)

    if game.black_long_castle():
        input.append(1)
    else:
        input.append(0)

    if game.white_short_castle():
        input.append(1)
    else:
        input.append(0)

    if game.black_short_castle():
        input.append(1)
    else:
        input.append(0)

    return input


def piece_as_value(piece):
    value = 0
    pieceside = piece[0]
    piecetype = piece[1]

    if piecetype == cc.Pawn:
        value = 1
    elif piecetype == cc.Knight:
        value = 2
    elif piecetype == cc.Bishop:
        value = 3
    elif piecetype == cc.Rook:
        value = 4
    elif piecetype == cc.Queen:
        value = 5
    elif piecetype == cc.King:
        value = 6

    if pieceside == cc.Black:
        value += 6

    return value


def game_classes():
    classes = [13 for a in range(0, 64)]
    classes += [2, 9, 2, 2, 2, 2]
    return classes