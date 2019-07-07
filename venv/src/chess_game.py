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
            return 1
        else:
            return 0

    def white_short_castle(self):
        if self.pieceMoveNums[7] + self.pieceMoveNums[4] == 0:
            return 1
        else:
            return 0

    def black_long_castle(self):
        if self.pieceMoveNums[16] + self.pieceMoveNums[20] == 0:
            return 1
        else:
            return 0

    def black_short_castle(self):
        if self.pieceMoveNums[23] + self.pieceMoveNums[20] == 0:
            return 1
        else:
            return 0

    def position_hash(self):
        phash = str(self.to_move) + \
                str(self.white_long_castle()) + \
                str(self.white_short_castle()) + \
                str(self.black_long_castle()) + \
                str(self.black_short_castle()) + \
                str(self.enPassant[0])

        for file in range(1, cc.H + 1):
            for rank in range(1, 9):
                pieceat = self.board[file][rank][0]

                if pieceat is not None:
                    piece = self.pieces[pieceat]
                    phash += cp.file_to_letter(file)
                    phash += rank
                    phash += piece[0]
                    phash += cp.piece_to_letter(piece[1])
        return phash

    def apply_move(self, move):

        if move.longCastleWhite or move.longCastleBlack or move.shortCastleWhite or move.shortCastleBlack:
            self.enPassant.appendleft(0)
            self.fiftyMoveRuleCount.appendleft(self.fiftyMoveRuleCount[0] + 1)
            if move.longCastleWhite:
                a1pce = self.board[cc.A][1][0]
                e1pce = self.board[cc.E][1][0]
                self.board[cc.A][1].appendleft(None)
                self.board[cc.E][1].appendleft(None)
                self.positions[a1pce].appendleft((cc.D, 1))
                self.positions[e1pce].appendleft((cc.C, 1))
                self.board[cc.D][1].appendleft(a1pce)
                self.board[cc.C][1].appendleft(e1pce)
                self.pieceMoveNums[e1pce] += 1
            elif move.longCastleBlack:
                a8pce = self.board[cc.A][8][0]
                e8pce = self.board[cc.E][8][0]
                self.board[cc.A][8].appendleft(None)
                self.board[cc.E][8].appendleft(None)
                self.positions[a8pce].appendleft((cc.D, 8))
                self.positions[e8pce].appendleft((cc.C, 8))
                self.board[cc.D][8].appendleft(a8pce)
                self.board[cc.C][8].appendleft(e8pce)
                self.pieceMoveNums[e8pce] += 1
            elif move.shortCastleWhite:
                h1pce = self.board[cc.H][1][0]
                e1pce = self.board[cc.E][1][0]
                self.board[cc.H][1].appendleft(None)
                self.board[cc.E][1].appendleft(None)
                self.positions[h1pce].appendleft((cc.F, 1))
                self.positions[e1pce].appendleft((cc.G, 1))
                self.board[cc.F][1].appendleft(h1pce)
                self.board[cc.G][1].appendleft(e1pce)
                self.pieceMoveNums[e1pce] += 1
            elif move.shortCastleBlack:
                h8pce = self.board[cc.H][8][0]
                e8pce = self.board[cc.E][8][0]
                self.board[cc.H][8].appendleft(None)
                self.board[cc.E][8].appendleft(None)
                self.positions[h8pce].appendleft((cc.F, 8))
                self.positions[e8pce].appendleft((cc.G, 8))
                self.board[cc.F][8].appendleft(h8pce)
                self.board[cc.G][8].appendleft(e8pce)
                self.pieceMoveNums[e8pce] += 1
        else:
            piecenum = self.board[move.fr[0]][move.fr[1]][0]
            self.board[move.fr[0]][move.fr[1]].appendleft(None)
            pieceside = self.pieces[piecenum][0]
            piecetype = self.pieces[piecenum][1]

            if piecetype != cc.Pawn:
                self.enPassant.appendleft(0)
                if not move.capture:
                    self.fiftyMoveRuleCount.appendleft(self.fiftyMoveRuleCount[0] + 1)
                    self.positions[piecenum].appendleft(move.to)
                    self.board[move.to[0]][move.to[1]].appendleft(piecenum)
                else:
                    self.fiftyMoveRuleCount.appendleft(0)
                    self.positions[piecenum].appendleft(move.to)
                    self.board[move.to[0]][move.to[1]].appendleft(piecenum)
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
                    self.board[move.to[0]][move.to[1]].appendleft(piecenum)
                elif move.capture and move.enPassant == 0:
                    self.positions[piecenum].appendleft(move.to)
                    self.board[move.to[0]][move.to[1]].appendleft(piecenum)
                else:
                    self.positions[piecenum].appendleft(move.to)
                    self.board[move.to[0]][move.to[1]].appendleft(piecenum)
                    if self.pieces[piecenum][0] == cc.White:
                        self.board[move.enPassant][5].appendleft(None)
                    elif self.pieces[piecenum][0] == cc.Black:
                        self.board[move.enPassant][4].appendleft(None)
                if move.promote is not None:
                    self.pieces[piecenum][1] = move.promote
        phash = self.position_hash()
        if phash not in self.repetitions:
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
        del move[-1]
        self.enPassant.popleft()
        self.fiftyMoveRuleCount.popleft()

        squares_changed = []
        pos_changed = []

        if move.longCastleWhite or move.longCastleBlack or move.shortCastleWhite or move.shortCastleBlack:
            if move.longCastleWhite:
                king = self.board[cc.C][1][0]
                self.pieceMoveNums[king] -= 1
                squares_changed += [(cc.A, 1), (cc.C, 1), (cc.D, 1), (cc.E, 1)]
                pos_changed += [0, 4]
            elif move.longCastleBlack:
                king = self.board[cc.C][8][0]
                self.pieceMoveNums[king] -= 1
                squares_changed += [(cc.A, 8), (cc.C, 8), (cc.D, 8), (cc.E, 8)]
                pos_changed += [7, 4]
            elif move.shortCastleWhite:
                king = self.board[cc.G][1][0]
                self.pieceMoveNums[king] -= 1
                squares_changed += [(cc.E, 1), (cc.F, 1), (cc.G, 1), (cc.H, 1)]
                pos_changed += [16, 20]
            elif move.shortCastleBlack:
                king = self.board[cc.G][8][0]
                self.pieceMoveNums[king] -= 1
                squares_changed += [(cc.E, 8), (cc.F, 8), (cc.G, 8), (cc.H, 8)]
                pos_changed += [23, 20]
        else:
            piece = self.board[move.to[0]][move.to[1]][0]
            squares_changed += [move.fr, move.to]
            self.pieceMoveNums[self.board[move.to[0]][move.to[1]]] -= 1
            pieceside = self.pieces[piece][0]
            pos_changed.append(piece)

            if move.capture and move.enPassant == 0:
                pos_changed.append(self.board[move.to[0][move.to[1]]][1])
            if move.enPassant != 0 and pieceside == cc.White:
                squares_changed.append((move.enPassant, 5))
                pos_changed.append(self.board[move.enPassant][5][1])
            elif move.enPassant != 0 and pieceside == cc.Black:
                squares_changed.append((move.enPassant, 4))
                pos_changed.append(self.board[move.enPassant][4][1])

            if move.promote is not None:
                self.pieces[piece][1] = cc.Pawn

        for sq in squares_changed:
            self.board[sq[0]][sq[1]].popleft()

        for pce in pos_changed:
            self.positions[pce].popleft()
