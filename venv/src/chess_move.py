
class ChessMove:
    def __init__(self, fr, to, capture = False, enpass = 0, promote = None, longcastlewhite = False, longcastleblack = False, shortcastlewhite = False, shortcastleblack = False):
        self.fr = fr
        self.to = to
        self.capture = capture
        self.enPassant = enpass
        self.promote = promote
        self.longCastleWhite = longcastlewhite
        self.longCastleBlack = longcastleblack
        self.shortCastleWhite = shortcastlewhite
        self.shortCastleBlack = shortcastleblack
