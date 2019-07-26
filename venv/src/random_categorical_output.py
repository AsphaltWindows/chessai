import chess.categorical_input as ci
import chess.chess_game as cg
import chess.chess_consts as cc
import chess.moves as m
import chess.game_state as gs

import random as rand
import sys

game_num = int(sys.argv[1])
directory = sys.argv[2]
white_win = 0
black_win = 0
draw = 0

whitewins = open(directory + "white_wins.games", "w")
blackwins = open(directory + "black_wins.games", "w")
draws = open(directory + "draws.games", "w")

print("Playing " + str(game_num) + " games")

for num in range(0, game_num):
    game = cg.ChessGame()
    position_inputs = []
    while gs.game_state(game) == cc.InProgress:
        legal_moves_pos = m.all_legal_moves(game, ci.game_as_input)
        idx = rand.choice(range(0, len(legal_moves_pos[0])))
        move = legal_moves_pos[0][idx]
        pos = (str(legal_moves_pos[1][idx])[1:-1])
        pos = pos.replace(',', '') + '\n'
        position_inputs.append(pos)
        game.apply_move(move)

    state = gs.game_state(game)

    out = "Game " + str(num) + ": "
    if state == cc.Draw:
        draw += 1
        for p in position_inputs:
            draws.write(p)
        print(out + "Draw")
    elif state == cc.WhiteWin:
        white_win += 1
        for p in position_inputs:
            whitewins.write(p)
        print(out + "White Wins")
    else:
        black_win += 1
        for p in position_inputs:
            blackwins.write(p)
        print(out + "Black Wins")

whitewins.close()
blackwins.close()
draws.close()

print(str(white_win) + " white wins, " + str(black_win) + " black wins, " + str(draw) + " draws.")

