class Chess:
    White = 0
    Black = 1

    WhiteWin = 0
    BlackWin = 1
    Draw = 2
    InProgress = 3

    @staticmethod
    def game_as_input(game):
        return None

    @staticmethod
    def new_game():
        return None

    @staticmethod
    def game_state(game):
        return Chess.InProgress

    @staticmethod
    def all_legal_moves(game, g_as_in):
        return None, None # Moves, Positions

    @staticmethod
    def apply_move(game, move):
        return None

    @staticmethod
    def move_to_string(move):
        return None

