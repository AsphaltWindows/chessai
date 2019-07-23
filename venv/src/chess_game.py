import chess_init as ci
import chess_consts as cc
import chess_parse as cp
import copy as copy
from collections import deque


class ChessGame:
    def __init__(self):
        self.to_move = cc.White
        self.pieces = copy.deepcopy(ci.initialPieces)
        self.positions = copy.deepcopy(ci.initialPositions)
        self.board = copy.deepcopy(ci.initialBoard)
        self.pieceMoveNums = copy.deepcopy(ci.initialPieceMoveNums)
        self.enPassant = deque([0])
        self.fiftyMoveRuleCount = deque([0])
        self.repetitions = {}
        self.movesPlayed = []

    def white_long_castle(self):
        if self.pieceMoveNums[0] + self.pieceMoveNums[4] == 0:
            return True
        else:
            return False

    def white_short_castle(self):
        if self.pieceMoveNums[7] + self.pieceMoveNums[4] == 0:
            return True
        else:
            return False

    def black_long_castle(self):
        if self.pieceMoveNums[16] + self.pieceMoveNums[20] == 0:
            return True
        else:
            return False

    def black_short_castle(self):
        if self.pieceMoveNums[23] + self.pieceMoveNums[20] == 0:
            return True
        else:
            return False

    def position_hash(self):
        phash = str(self.to_move) + \
                str(self.white_long_castle()) + \
                str(self.white_short_castle()) + \
                str(self.black_long_castle()) + \
                str(self.black_short_castle()) + \
                str(self.enPassant[0])

        for file in range(1, cc.H + 1):
            for rank in range(1, 9):
                pieceat = self.board[rank][file][0]

                if pieceat is not None:
                    piece = self.pieces[pieceat]
                    phash += cp.file_to_letter(file)
                    phash += str(rank)
                    phash += str(piece[0])
                    phash += cp.piece_to_letter(piece[1])
        return phash

    def apply_move(self, move):
        if move.longCastleWhite or move.longCastleBlack or move.shortCastleWhite or move.shortCastleBlack:
            self.enPassant.appendleft(0)
            self.fiftyMoveRuleCount.appendleft(self.fiftyMoveRuleCount[0] + 1)
            if move.longCastleWhite:
                king = 4
                rook = 0
                self.board[1][cc.A].appendleft(None)
                self.board[1][cc.E].appendleft(None)
                self.positions[rook].appendleft((cc.D, 1))
                self.positions[king].appendleft((cc.C, 1))
                self.board[1][cc.D].appendleft(rook)
                self.board[1][cc.C].appendleft(king)
                self.pieceMoveNums[king] += 1
            elif move.longCastleBlack:
                king = 20
                rook = 16
                self.board[8][cc.A].appendleft(None)
                self.board[8][cc.E].appendleft(None)
                self.positions[rook].appendleft((cc.D, 8))
                self.positions[king].appendleft((cc.C, 8))
                self.board[8][cc.D].appendleft(rook)
                self.board[8][cc.C].appendleft(king)
                self.pieceMoveNums[king] += 1
            elif move.shortCastleWhite:
                king = 4
                rook = 7
                self.board[1][cc.H].appendleft(None)
                self.board[1][cc.E].appendleft(None)
                self.positions[rook].appendleft((cc.F, 1))
                self.positions[king].appendleft((cc.G, 1))
                self.board[1][cc.F].appendleft(rook)
                self.board[1][cc.G].appendleft(king)
                self.pieceMoveNums[king] += 1
            elif move.shortCastleBlack:
                king = 20
                rook = 23
                self.board[8][cc.H].appendleft(None)
                self.board[8][cc.E].appendleft(None)
                self.positions[rook].appendleft((cc.F, 8))
                self.positions[king].appendleft((cc.G, 8))
                self.board[8][cc.F].appendleft(rook)
                self.board[8][cc.G].appendleft(king)
                self.pieceMoveNums[king] += 1
        else:
            piecenum = self.board[move.fr[1]][move.fr[0]][0]
            self.pieceMoveNums[piecenum] += 1
            self.board[move.fr[1]][move.fr[0]].appendleft(None)
            pieceside = self.pieces[piecenum][0]
            piecetype = self.pieces[piecenum][1]

            if piecetype != cc.Pawn:
                self.enPassant.appendleft(0)
                if not move.capture:
                    self.fiftyMoveRuleCount.appendleft(self.fiftyMoveRuleCount[0] + 1)
                    self.positions[piecenum].appendleft(move.to)
                    self.board[move.to[1]][move.to[0]].appendleft(piecenum)
                else:
                    self.fiftyMoveRuleCount.appendleft(0)
                    captured = self.board[move.to[1]][move.to[0]][0]
                    self.positions[captured].appendleft(None)
                    self.positions[piecenum].appendleft(move.to)
                    self.board[move.to[1]][move.to[0]].appendleft(piecenum)
            else:
                self.fiftyMoveRuleCount.appendleft(0)
                if pieceside == cc.White and move.fr[1] == 2 and move.to[1] == 4:
                    self.enPassant.appendleft(move.fr[0])
                elif pieceside == cc.Black and move.fr[1] == 7 and move.to[1] == 5:
                    self.enPassant.appendleft(move.fr[0])
                else:
                    self.enPassant.appendleft(0)

                if not move.capture:
                    self.positions[piecenum].appendleft(move.to)
                    self.board[move.to[1]][move.to[0]].appendleft(piecenum)
                elif move.capture and move.enPassant == 0:
                    captured = self.board[move.to[1]][move.to[0]][0]
                    self.positions[piecenum].appendleft(move.to)
                    self.positions[captured].appendleft(None)
                    self.board[move.to[1]][move.to[0]].appendleft(piecenum)
                else:
                    self.positions[piecenum].appendleft(move.to)
                    self.board[move.to[1]][move.to[0]].appendleft(piecenum)
                    if self.pieces[piecenum][0] == cc.White:
                        captured = self.board[5][move.enPassant][0]
                        self.positions[captured].appendleft(None)
                        self.board[5][move.enPassant].appendleft(None)
                    elif self.pieces[piecenum][0] == cc.Black:
                        captured = self.board[4][move.enPassant][0]
                        self.positions[captured].appendleft(None)
                        self.board[4][move.enPassant].appendleft(None)
                if move.promote is not None:
                    self.pieces[piecenum] = (self.pieces[piecenum][0], move.promote)

        if self.to_move == cc.White:
            self.to_move = cc.Black
        else:
            self.to_move = cc.White

        phash = self.position_hash()

        if phash in self.repetitions:
            self.repetitions[phash] += 1
        else:
            self.repetitions.update({phash: 1})

        self.movesPlayed.append(move)

    def unapply_move(self):
        phash = self.position_hash()

        if self.repetitions[phash] == 1:
            del self.repetitions[phash]
        else:
            self.repetitions[phash] -= 1

        move = self.movesPlayed[-1]


        self.movesPlayed.pop()
        self.enPassant.popleft()
        self.fiftyMoveRuleCount.popleft()

        squares_changed = []
        pos_changed = []

        if move.longCastleWhite or move.longCastleBlack or move.shortCastleWhite or move.shortCastleBlack:
            if move.longCastleWhite:
                king = self.board[1][cc.C][0]
                self.pieceMoveNums[king] -= 1
                squares_changed += [(cc.A, 1), (cc.C, 1), (cc.D, 1), (cc.E, 1)]
                pos_changed += [0, 4]
            elif move.longCastleBlack:
                king = self.board[8][cc.C][0]
                self.pieceMoveNums[king] -= 1
                squares_changed += [(cc.A, 8), (cc.C, 8), (cc.D, 8), (cc.E, 8)]
                pos_changed += [16, 20]
            elif move.shortCastleWhite:
                king = self.board[1][cc.G][0]
                self.pieceMoveNums[king] -= 1
                squares_changed += [(cc.E, 1), (cc.F, 1), (cc.G, 1), (cc.H, 1)]
                pos_changed += [7, 4]
            elif move.shortCastleBlack:
                king = self.board[8][cc.G][0]
                self.pieceMoveNums[king] -= 1
                squares_changed += [(cc.E, 8), (cc.F, 8), (cc.G, 8), (cc.H, 8)]
                pos_changed += [23, 20]
        else:
            piece = self.board[move.to[1]][move.to[0]][0]
            squares_changed += [move.fr, move.to]
            self.pieceMoveNums[self.board[move.to[1]][move.to[0]][0]] -= 1
            pieceside = self.pieces[piece][0]
            pos_changed.append(piece)

            if move.capture and move.enPassant == 0:
                pos_changed.append(self.board[move.to[1]][move.to[0]][1])
            if move.enPassant != 0 and pieceside == cc.White:
                squares_changed.append((move.enPassant, 5))
                pos_changed.append(self.board[5][move.enPassant][1])
            elif move.enPassant != 0 and pieceside == cc.Black:
                squares_changed.append((move.enPassant, 4))
                pos_changed.append(self.board[4][move.enPassant][1])

            if move.promote is not None:
                self.pieces[piece] = (self.pieces[piece][0], cc.Pawn)

        for sq in squares_changed:
            self.board[sq[1]][sq[0]].popleft()

        for pce in pos_changed:
            try:
                self.positions[pce].popleft()
            except:
                print("Popping position for piece: " + str(pce))

        if self.to_move == cc.White:
            self.to_move = cc.Black
        else:
            self.to_move = cc.White

    def print_board(self):
        board_str = u''
        file_str = u'    a   b   c   d   e   f   g   h\n'
        rank_sep = u'  +---+---+---+---+---+---+---+---+\n'
        board_str += file_str
        board_str += rank_sep
        for rank in range(8, 0, -1):
            board_str += str(rank)
            board_str += u' |'
            for file in range(cc.A, cc.H + 1):
                board_str += u' '
                piecenum = self.board[rank][file][0]
                if piecenum is None:
                    board_str += u' '
                else:
                    piecetype = self.pieces[piecenum][1]
                    pieceside = self.pieces[piecenum][0]

                    if piecetype == cc.Pawn and pieceside == cc.White:
                        board_str += u'\u265F'
                    elif piecetype == cc.Knight and pieceside == cc.White:
                        board_str += u'\u265E'
                    elif piecetype == cc.Bishop and pieceside == cc.White:
                        board_str += u'\u265D'
                    elif piecetype == cc.Rook and pieceside == cc.White:
                        board_str += u'\u265C'
                    elif piecetype == cc.Queen and pieceside == cc.White:
                        board_str += u'\u265B'
                    elif piecetype == cc.King and pieceside == cc.White:
                        board_str += u'\u265A'
                    elif piecetype == cc.Pawn and pieceside == cc.Black:
                        board_str += u'\u2659'
                    elif piecetype == cc.Knight and pieceside == cc.Black:
                        board_str += u'\u2658'
                    elif piecetype == cc.Bishop and pieceside == cc.Black:
                        board_str += u'\u2657'
                    elif piecetype == cc.Rook and pieceside == cc.Black:
                        board_str += u'\u2656'
                    elif piecetype == cc.Queen and pieceside == cc.Black:
                        board_str += u'\u2655'
                    elif piecetype == cc.King and pieceside == cc.Black:
                        board_str += u'\u2654'
                board_str += ' |'
            board_str += u' '
            board_str += str(rank)
            board_str += u'\n'
            board_str += rank_sep
        board_str += file_str
        return board_str

