import sys
import os
import numpy as np
import random as rand

import chess.categorical_input as ci
import chess.chess_consts as cc
import chess.chess_game as cg
import chess.game_state as gs
import chess.moves as m
import chess.move_strings as ms
import models.categorical_naive_bayes as cnb
import models.cnb_c as cnb_c
import models.clustered_bayes as cb
import models.k_network_bayes as knb
import models.hhcb as hhcb
import models.hhcbsl as hhcbsl
import chess.categorical_input as ci

model_type = sys.argv[1]
selfplay_dir = sys.argv[2]
model_version = int(sys.argv[3])
game_num = int(sys.argv[4])
round_num = int(sys.argv[5])
model_args = list(map(int, sys.argv[6].split(",")))

if model_type == "nb":
    if model_version == 0:
        model = cnb_c.CNB_C(3, ci.game_classes())
    else:
        model = cnb_c.CNB_C.load_model2(selfplay_dir + "/nb" + str(model_version) + ".model")
    train_model = model.train_batch
    store_model = model.store_model2

elif model_type == "cb":
    if model_version == 0:
        cluster_num = model_args[0]
        init_modes = []
        model = cb.ClusteredBayes(init_modes, 3, cluster_num, ci.game_classes())
    else:
        model = cb.ClusteredBayes.load_model2(selfplay_dir + "/cb" + str(model_version) + ".model")
    train_model = model.train_batch
    store_model = model.store_model2

elif model_type == "knb":
    if model_version == 0:
        cluster_num = model_args[0]
        node_num = model_args[1]
        layer_num = model_args[2]
        model = knb.KNetworkBayes(3, ci.game_classes(), cluster_num, node_num, layer_num)
    else:
        model = knb.KNetworkBayes.load_model(selfplay_dir + "/knb" + str(model_version) + ".model")
    train_model = model.train_batch
    store_model = model.store_model

elif model_type == "hhcbsl":
    if model_version == 0:
        cluster_num = model_args[0]
        limit = model_args[1]
        alpha = model_args[2]
        model = hhcbsl.HierarchicalHistogramClusteredBayesSizeLimited(ci.game_classes(), cluster_num, 3, limit, alpha)
    else:
        model = hhcbsl.HierarchicalHistogramClusteredBayesSizeLimited.load_model2(selfplay_dir + "/hhcbsl" + str(model_version) + ".model")

    train_model = model.train_model
    store_model = model.store_model2


def select_position(positions, play_as):
    weights = []

    for idx, pos in enumerate(positions):
        # print(pos)
        win_n = 2 ** pos[play_as]
        draw_n = 2 ** pos[cc.Draw]

        if play_as == cc.White:
            loss_n = 2 ** pos[cc.Black]
        else:
            loss_n = 2 ** pos[cc.White]
        # weights.append((win_n + draw_n / 2)**3)
        weights.append(((win_n + (draw_n / 2)) / (win_n + loss_n + draw_n)) ** 4)
        # if idx > 0:
        #     weights[idx] += weights[idx - 1]

    # return np.argmax(weights)
    return rand.choices(range(0, len(positions)), weights)[0]

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

move_idx = lambda m, t: select_move(model, t, m)

at_round = 0

resultsFile = open(selfplay_dir + "/results.txt", "w")

while at_round < round_num:
    model_version += 1

    whitewin_num = 0
    blackwin_num = 0
    draw_num = 0

    print("Beginning round " + str(at_round))

    labels = []
    data = []

    for n in range(0, game_num):
        game = cg.ChessGame()
        position_inputs = []

        while gs.game_state(game) == cc.InProgress:

            all_moves = m.all_legal_moves(game, ci.game_as_input)
            encoded_moves = all_moves[1]
            actual_moves = all_moves[0]
            idx = move_idx(encoded_moves, game.to_move)
            position_inputs.append(encoded_moves[idx])
            game.apply_move(actual_moves[idx])

        position_inputs.append(ci.game_as_input(game))

        state = gs.game_state(game)

        data += position_inputs

        if state == cc.Draw:
            labels += [cc.Draw for pos in position_inputs]
            draw_num += 1
        elif state == cc.WhiteWin:
            labels += [cc.WhiteWin for pos in position_inputs]
            whitewin_num += 1
        else:
            labels += [cc.BlackWin for pos in position_inputs]
            blackwin_num += 1

    print("Training model on games of round " + str(at_round))

    train_model(
        labels,
        data
    )

    resultsFile.write("round " + str(at_round) + " white wins: " + str(whitewin_num) + " black wins: " + str(blackwin_num) + " draws : " + str(draw_num) + "\n")

    at_round += 1

    store_model(selfplay_dir + "/" + model_type + str(model_version) + ".model")

resultsFile.close()


