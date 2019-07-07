
class ChessMove:
    def __init__(self, at, to, capture = False, enpass = 0, promote = None, longcastlewhite = False, longcastleblack = False, shortcastlewhite = False, shortcastleblack = False):
        self.at = at
        self.to = to
        self.capture = capture
        self.enPassant = enpass
        self.promoteTo = promote
        self.longCastleWhite = longcastlewhite
        self.longCastleBlack = longcastleblack
        self.shortCastleWhite = shortcastlewhite
        self.shortCastleBlack = shortcastleblack
