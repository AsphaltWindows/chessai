import sys

import models.categorical_naive_bayes as cnb
import models.cnb_c as cnb_c
import models.bdt_c as bdt
import models.clustered_bayes as cb
import models.k_network_bayes as knb
import models.hhcb as hhcb
import models.hhcbsl as hhcbsl
import chess.categorical_input as ci

model_type = sys.argv[1]
model_dir = sys.argv[2]
model_version = int(sys.argv[3])
games_dir = sys.argv[4]
model_args = list(map(int, sys.argv[5].split(",")))

whitewinsfile = open(games_dir + "white_wins.games", "r")
blackwinsfile = open(games_dir + "black_wins.games", "r")
drawsfile = open(games_dir + "draws.games", "r")

whitewins = [list(map(int, line.split(" "))) for line in whitewinsfile.readlines()]
blackwins = [list(map(int, line.split(" "))) for line in blackwinsfile.readlines()]
draws = [list(map(int, line.split(" "))) for line in drawsfile.readlines()]

if model_type == "nb":
    if model_version == 0:
        model = cnb_c.CNB_C(3, ci.game_classes())
    else:
        model = cnb_c.CNB_C.load_model2(model_dir + "/nb" + str(model_version) + ".model")

    model.train_batch([0 for i in whitewins] + [1 for i in blackwins] + [2 for i in draws], whitewins + blackwins + draws)

    model.store_model2(model_dir + "/nb" + str(model_version + 1) + ".model")

elif model_type == "cb":
    if model_version == 0:
        cluster_num = model_args[0]
        init_modes = []
        model = cb.ClusteredBayes(init_modes, 3, cluster_num, ci.game_classes())
    else:
        model = cb.ClusteredBayes.load_model2(model_dir + "/cb" + str(model_version) + ".model")

    model.train_batch([0 for i in whitewins] + [1 for i in blackwins] + [2 for i in draws], whitewins + blackwins + draws)

    model.store_model2(model_dir + "/cb" + str(model_version + 1) + ".model")

elif model_type == "knb":
    if model_version == 0:
        cluster_num = model_args[0]
        node_num = model_args[1]
        layer_num = model_args[2]
        model = knb.KNetworkBayes(3, ci.game_classes(), cluster_num, node_num, layer_num)
    else:
        model = knb.KNetworkBayes.load_model(model_dir + "/knb" + str(model_version) + ".model")

    model.train_batch([0 for i in whitewins] + [1 for i in blackwins] + [2 for i in draws], whitewins + blackwins + draws)

    model.store_model(model_dir + "/knb" + str(model_version + 1) + ".model")

elif model_type == "hhcb":
    if model_version == 0:
        cluster_num = model_args[0]
        tree_depth = model_args[1]
        alpha = model_args[2]
        model = hhcb.HierarchicalHistogramClusteredBayes(ci.game_classes(), cluster_num, 3, tree_depth, alpha)

        model.train_full([0 for i in whitewins] + [1 for i in blackwins] + [2 for i in draws], whitewins + blackwins + draws)
    else:
        model = hhcb.HierarchicalHistogramClusteredBayes.load_model2(model_dir + "/hhcb" + str(model_version) + ".model")

        model.train_incremental([0 for i in whitewins] + [1 for i in blackwins] + [2 for i in draws], whitewins + blackwins + draws)

    model.store_model2(model_dir + "/hhcb" + str(model_version + 1) + ".model")
elif model_type == "hhcbsl":
    if model_version == 0:
        cluster_num = model_args[0]
        limit = model_args[1]
        alpha = model_args[2]
        model = hhcbsl.HierarchicalHistogramClusteredBayesSizeLimited(ci.game_classes(), cluster_num, 3, limit, alpha)

        model.train_model([0 for i in whitewins] + [1 for i in blackwins] + [2 for i in draws], whitewins + blackwins + draws)
    else:
        model = hhcbsl.HierarchicalHistogramClusteredBayesSizeLimited.load_model2(model_dir + "/hhcbsl" + str(model_version) + ".model")

        model.train_model([0 for i in whitewins] + [1 for i in blackwins] + [2 for i in draws], whitewins + blackwins + draws)

    model.store_model2(model_dir + "/hhcbsl" + str(model_version + 1) + ".model")
elif model_type == "bdt":
    print("training bdt model")
    if model_version == 0:
        branch_factor = model_args[0]
        split_threshold = model_args[1]
        split_limit = model_args[2]
        forget_factor = model_args[3]
        alpha = model_args[4]
        model = bdt.BDT_C(ci.game_classes(), 3, branch_factor, split_threshold, split_limit, alpha)

        print("created model")
        model.train_batch([[1,0,0] for i in whitewins] + [[0,1,0] for i in blackwins] + [[0,0,1] for i in draws], whitewins + blackwins + draws)
    else:
        model = bdt.model_from_file(model_dir + "/bdt" + str(model_version) + ".model")

        model.train_batch([[1,0,0] for i in whitewins] + [[0,1,0] for i in blackwins] + [[0,0,1] for i in draws], whitewins + blackwins + draws)

    model.model_to_file(model_dir + "/bdt" + str(model_version + 1) + ".model")


whitewinsfile.close()
blackwinsfile.close()
drawsfile.close()






