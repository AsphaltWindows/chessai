import models.categorical_naive_bayes as nb
import models.cnb_c as nb_c
import models.hhcb as hhcb
import models.hhcbsl as hhcbsl
import models.k_network_bayes as knb
import models.bdt_c as bdt
import models.clustered_bayes as cb
import chess.categorical_input as ci
import chess.chess_consts as cc
import chess.chess_game as cg
import chess.game_state as gs
import chess.moves as m
import chess.move_strings as ms
import random as rand
import numpy as np

import sys


def select_position(positions, play_as):
    weights = []

    for idx, pos in enumerate(positions):
        # print(pos)
        win_n = pos[play_as]
        draw_n = pos[cc.Draw]

        if play_as == cc.White:
            loss_n = pos[cc.Black]
        else:
            loss_n = pos[cc.White]
        # weights.append(((win_n + (draw_n / 2)) / (win_n + loss_n + draw_n)) ** 4)
        weights.append((win_n + (draw_n / 2)) / (win_n + loss_n + draw_n))
        # if idx > 0:
        #     weights[idx] += weights[idx - 1]

    return np.argmax(weights)
    # return rand.choices(range(0, len(positions)), weights)[0]

    # chance_to_win = False
    # best_idx = 0
    # best_win = 0
    # best_draw = 0
    #
    # for idx, pos in enumerate(positions):
    #     total = sum(pos)
    #     win_n = pos[play_as] / total
    #     draw_n = pos[cc.Draw] / total
    #     lose_n = 1 - (win_n - draw_n)
    #
    #     if win_n > lose_n:
    #         chance_to_win = True
    #
    #     if chance_to_win and best_win < win_n:
    #         best_win = win_n
    #         best_idx = idx
    #
    #     if not chance_to_win and best_draw < draw_n:
    #         best_draw = draw_n
    #         best_idx = idx
    #
    # return best_idx


def select_move(mod, tomov, moves):
    evals = [mod.predict_class(pos) for pos in moves]
    return select_position(evals, tomov)


game_num = int(sys.argv[1])
game_dir = sys.argv[2]

model1_type = sys.argv[3]
model1_dir = sys.argv[4]
model1_version = int(sys.argv[5])

model2_type = sys.argv[6]
model2_dir = sys.argv[7]
model2_version = int(sys.argv[8])

player1 = None
player2 = None

if model1_type == "rand":
    player1 = lambda m, t: rand.randrange(0, len(m))
elif model1_type == "nb":
    model1 = nb_c.CNB_C.load_model2(model1_dir + "/" + model1_type + str(model1_version) + ".model")
    # model1 = nb.CategoricalNaiveBayes.load_model2(model1_dir + "/" + model1_type + str(model1_version) + ".model")
    player1 = lambda m, t: select_move(model1, t, m)
elif model1_type == "cb":
    model1 = cb.ClusteredBayes.load_model2(model1_dir + "/" + model1_type + str(model1_version) + ".model")
    player1 = lambda m, t: select_move(model1, t, m)
elif model1_type == "knb":
    model1 = knb.KNetworkBayes.load_model2(model1_dir + "/" + model1_type + str(model1_version) + ".model")
    player1 = lambda m, t: select_move(model1, t, m)
elif model1_type == "hhcb":
    model1 = hhcb.HierarchicalHistogramClusteredBayes.load_model2(model1_dir + "/" + model1_type + str(model1_version) + ".model")
    player1 = lambda m, t: select_move(model1, t, m)
elif model1_type == "hhcbsl":
    model1 = hhcbsl.HierarchicalHistogramClusteredBayesSizeLimited.load_model2(model1_dir + "/" + model1_type + str(model1_version) + ".model")
    player1 = lambda m, t: select_move(model1, t, m)
elif model1_type == "bdt":
    model1 = bdt.BDT_C.model_from_file(model1_dir + "/" + model1_type + str(model1_version) + ".model", ci.game_classes(), 3)
    player1 = lambda m, t: select_move(model1, t, m)

if model2_type == "rand":
    player2 = lambda m, t: rand.randrange(0, len(m))
elif model2_type == "nb":
    model2 = nb_c.CNB_C.load_model2(model2_dir + "/" + model2_type + str(model2_version) + ".model")
    # model2 = nb.CategoricalNaiveBayes.load_model2(model2_dir + "/" + model2_type + str(model2_version) + ".model")
    player2 = lambda m, t: select_move(model2, t, m)
elif model2_type == "cb":
    model2 = cb.ClusteredBayes.load_model(model2_dir + "/" + model2_type + str(model2_version) + ".model")
    player2 = lambda m, t: select_move(model2, t, m)
elif model2_type == "knb":
    model2 = knb.KNetworkBayes.load_model(model2_dir + "/" + model2_type + str(model2_version) + ".model")
    player2 = lambda m, t: select_move(model2, t, m)
elif model2_type == "hhcb":
    model2 = hhcb.HierarchicalHistogramClusteredBayes.load_model2(model2_dir + "/" + model2_type + str(model2_version) + ".model")
    player2 = lambda m, t: select_move(model2, t, m)
elif model2_type == "hhcbsl":
    model2 = hhcbsl.HierarchicalHistogramClusteredBayesSizeLimited.load_model2(model2_dir + "/" + model2_type + str(model2_version) + ".model")
    player2 = lambda m, t: select_move(model2, t, m)
elif model2_type == "bdt":
    model2 = bdt.BDT_C.model_from_file(model2_dir + "/" + model2_type + str(model2_version) + ".model", ci.game_classes(), 3)
    player2 = lambda m, t: select_move(model2, t, m)

player1_side = cc.White

draw_num = 0
win_num = 0
loss_num = 0

whitewins = open(game_dir + "white_wins.games", "w")
blackwins = open(game_dir + "black_wins.games", "w")
draws = open(game_dir + "draws.games", "w")
human = open(game_dir + "human.games", "w")

for n in range(0, game_num):

    game = cg.ChessGame()
    position_inputs = []
    human_game = ""
    move_num = 0

    while gs.game_state(game) == cc.InProgress:
        to_move = game.to_move

        if to_move == cc.White:
            move_num += 1
            human_game += (str(move_num) + ". ")

        all_moves = m.all_legal_moves(game, ci.game_as_input)
        encoded_moves = all_moves[1]
        actual_moves = all_moves[0]
        idx = 0

        if to_move == player1_side:
            idx = player1(encoded_moves, to_move)

        else:
            idx = player2(encoded_moves, to_move)

        pos = (str(all_moves[1][idx])[1:-1])
        pos = pos.replace(',', '') + '\n'
        position_inputs.append(pos)
        human_game += ms.move_to_string(actual_moves[idx], game)
        human_game += " "
        game.apply_move(actual_moves[idx])

    state = gs.game_state(game)
    out = "Game " + str(n) + ": "

    if state == cc.Draw:
        draw_num += 1
        human_game += "1/2-1/2\n\n"
        for p in position_inputs:
            draws.write(p)
        print(out + "Draw")
    elif state == player1_side:
        win_num += 1
        print(out + model1_type + str(model1_version) + " Wins")
        if player1_side == cc.White:
            for p in position_inputs:
                whitewins.write(p)
            human_game += "1-0\n\n"
        else:
            for p in position_inputs:
                blackwins.write(p)
            human_game += "0-1\n\n"
    else:
        loss_num += 1
        print(out + model2_type + str(model2_version) + " Wins")
        if player1_side == cc.White:
            for p in position_inputs:
                blackwins.write(p)
            human_game += "0-1\n\n"
        else:
            for p in position_inputs:
                whitewins.write(p)
            human_game += "1-0\n\n"

    if player1_side == cc.White:
        player1_side = cc.Black
    else:
        player1_side = cc.White

    human.write(human_game)

results = model1_type + str(model1_version) + " vs " + model2_type + str(model2_version) + ": +" + str(win_num) + "-" + str(loss_num) + "=" + str(draw_num)
human.write(results)
print(results)

whitewins.close()
blackwins.close()
draws.close()
human.close()

