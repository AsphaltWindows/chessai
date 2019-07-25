import models.categorical_naive_bayes as cnb
import chess.categorical_input as ci
import chess.chess_consts as cc
import chess.chess_game as cg
import chess.game_state as gs
import chess.moves as m
import random as rand


def select_position(positions, play_as):
    chance_to_win = False
    best_idx = 0
    best_win = 0
    best_draw = 0

    for idx, pos in enumerate(positions):
        total = sum(pos)
        win_n = pos[play_as] / total
        draw_n = pos[cc.Draw] / total
        lose_n = 1 - (win_n - draw_n)

        if win_n > lose_n:
            chance_to_win = True

        if chance_to_win and best_win < win_n:
            best_win = win_n
            best_idx = idx

        if not chance_to_win and best_draw < draw_n:
            best_draw = draw_n
            best_idx = idx

    # if chance_to_win:
    #     print("playing for win")
    # else:
    #     print("playing for draw")

    return best_idx


def play_move(mod, gam):
    moves = m.all_legal_moves(gam, ci.game_as_input)
    evals = [mod.predict_cat(pos) for pos in moves[1]]
    idx = select_position(evals, gam.to_move)
    gam.apply_move(moves[0][idx])


# Train bayesian model
model = cnb.CategoricalNaiveBayes(3, ci.game_classes())
directory = "/home/iv/dev/chessai/games/random/"
whitewinsfile = open(directory + "white_wins.games", "r")
blackwinsfile = open(directory + "black_wins.games", "r")
drawsfile = open(directory + "draws.games", "r")

whitewins = [(0, map(int, line.split(" "))) for line in whitewinsfile.readlines()]
blackwins = [(1, map(int, line.split(" "))) for line in blackwinsfile.readlines()]
draws = [(2, map(int, line.split(" "))) for line in drawsfile.readlines()]

model.train_batch(whitewins + blackwins + draws)

whitewinsfile.close()
blackwinsfile.close()
drawsfile.close()

num_games = 200
player = lambda g: play_move(model, g)
opponent = lambda g: g.apply_move(rand.choice(m.all_legal_moves(g)[0]))

current_side = cc.White

draw_num = 0
win_num = 0
loss_num = 0

for n in range(0, num_games):

    game = cg.ChessGame()

    while gs.game_state(game) == cc.InProgress:
        if game.to_move == current_side:
            player(game)
        else:
            opponent(game)

    state = gs.game_state(game)

    if state == cc.Draw:
        draw_num += 1
    elif state == current_side:
        win_num += 1
    else:
        loss_num += 1

    if current_side == cc.White:
        current_side = cc.Black
    else:
        current_side = cc.White

print("+" + str(win_num) + "-" + str(loss_num) + "=" + str(draw_num))
