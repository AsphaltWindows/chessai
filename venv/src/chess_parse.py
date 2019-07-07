import chess_consts as cc


letter_to_fl = {
    "a": cc.A,
    "b": cc.B,
    "c": cc.C,
    "d": cc.D,
    "e": cc.E,
    "f": cc.F,
    "g": cc.G,
    "h": cc.H
}

fl_to_letter = {
    cc.A: "a",
    cc.B: "b",
    cc.C: "c",
    cc.D: "d",
    cc.E: "e",
    cc.F: "f",
    cc.G: "g",
    cc.H: "H",
}

letter_to_pce = {
    "P": cc.Pawn,
    "N": cc.Knight,
    "B": cc.Bishop,
    "R": cc.Rook,
    "Q": cc.Queen,
    "K": cc.King
}

pce_to_letter = {
    cc.Pawn: "P",
    cc.Knight: "N",
    cc.Bishop: "B",
    cc.Rook: "R",
    cc.Queen: "Q",
    cc.King: "K",
}


def letter_to_file(letter):
    return letter_to_fl[letter]


def letter_to_piece(letter):
    return letter_to_pce[letter]


def file_to_letter(file):
    return fl_to_letter[file]


def piece_to_letter(piece):
    return pce_to_letter[piece]

# def string_to_move(board, move):
#
#
#
# def move_to_string(board, move):
#     fileFrom = letter_to_file(move[1]) #     rankFrom = int(move[2])
#     fileTo = letter_to_file(move[4])
#     rankTo = int(move[5])
#     capture = board[fileTo][rankTo]
