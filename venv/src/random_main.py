import chess_game as cg
import chess_parse as cp
import game_state as gs
import chess_consts as cc
import move_strings as ms
import moves as m
import random as rand

directory = "/home/iv/dev/chessai/games/random/"
game_num = 200
white_win = 0
black_win = 0
draw = 0

for num in range(0, game_num):
    f = open(directory + str(num) + ".game", "w")
    game = cg.ChessGame()
    move_num = 1
    game_str = ""
    while gs.game_state(game) == cc.InProgress:
        legal_moves = m.all_legal_moves(game)
        move = rand.choice(legal_moves)

        if game.to_move == cc.White:
            game_str += str(move_num)
            game_str += ". "
            move_num += 1

        game_str += ms.move_to_string(move, game)
        game_str += " "
        game.apply_move(move)
    state = gs.game_state(game)

    if state == cc.Draw:
        draw += 1
    elif state == cc.WhiteWin:
        white_win += 1
    else:
        black_win += 1

    f.write(game.print_board())
    f.write(game_str)
    f.write(cp.state_to_string(gs.game_state(game)))
    f.close()

print(str(white_win) + " white wins, " + str(black_win) + " black wins, " + str(draw) + " draws.")

