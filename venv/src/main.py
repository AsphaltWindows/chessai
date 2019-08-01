import chess.chess_game as cg
import chess.chess_parse as cp
import chess.game_state as gs
import chess.chess_consts as cc
import chess.move_strings as ms
import chess.moves as m
import chess.categorical_input as ci
import models.cnb_c as cnbc
import models.km_c as kmc
import models.k_modes as km
import models.k_network_bayes as knb
import models.clustered_bayes as cb


games_dir = "/home/iv/dev/chessai/games/random_random/"

whitewinsfile = open(games_dir + "white_wins.games", "r")
# blackwinsfile = open(games_dir + "black_wins.games", "r")
# drawsfile = open(games_dir + "draws.games", "r")

whitewins = [list(map(int, line.split(" "))) for line in whitewinsfile.readlines()]
# blackwins = [list(map(int, line.split(" "))) for line in blackwinsfile.readlines()]
# draws = [list(map(int, line.split(" "))) for line in drawsfile.readlines()]

clustering = kmc.KM_C([whitewins[0], whitewins[1], whitewins[2]], 3, ci.game_classes())
classifier = cnbc.CNB_C(3, ci.game_classes())

py_clust = km.KModes([whitewins[0], whitewins[1], whitewins[2]], 3, ci.game_classes())

clustering.train_batch(whitewins[0:1000])
py_clust.train_batch(whitewins[0:1000])

# clustering = kmc.KM_C([[1, 1], [0, 0]], 2, [2,2])
#
# py_clust = km.KModes([[1, 1], [0, 0]], 2, [2,2])

# print("about to start training classification")
# classifier.train_batch([1 for w in whitewins], whitewins)
# print("about to start training c clustering")
# clustering.train_batch([[1,0],[1,0],[0,1],[0,1]])
# print("about to start training python clustering")
# py_clust.train_batch([[1,0],[1,0],[0,1],[0,1]])

whitewinsfile.close()
# blackwinsfile.close()
# drawsfile.close()

# knmodel = knb.KNetworkBayes.load_model("/home/iv/dev/chessai/models/k_network_bayes/knb1.model")
# knmodel.store_model2("/home/iv/dev/chessai/models/k_network_bayes/knb2.model")
#
# knmodel = knb.KNetworkBayes.load_model2("/home/iv/dev/chessai/models/k_network_bayes/knb2.model")
# knmodel.store_model2("/home/iv/dev/chessai/models/k_network_bayes/knb3.model")

# cbmodel = cb.ClusteredBayes.load_model("/home/iv/dev/chessai/models/clustered_bayes/cb3.model")
# cbmodel.store_model2("/home/iv/dev/chessai/models/clustered_bayes/cb4.model")
#
# cbmodel = cb.ClusteredBayes.load_model2("/home/iv/dev/chessai/models/clustered_bayes/cb4.model")
# cbmodel.store_model2("/home/iv/dev/chessai/models/clustered_bayes/cb5.model")

game = cg.ChessGame()

print (gs.game_state(game))

while gs.game_state(game) == cc.InProgress:
    board = game.print_board()
    print(board)
    legal_moves = m.all_legal_moves(game)[0]
    print("number of legal moves: " + str(len(legal_moves)))
    print(game.enPassant)
    # if len(legal_moves) == 1:
    #     m = legal_moves[0]
    #     print(legal_moves)
    #     print("legal move  " + n + ": from: " + cp.file_to_letter(m.fr[0]) + str(m.fr[1]) + " to: " + cp.file_to_letter(m.to[0]) + str(m.to[1]) + "capture: " + str(m.capture))
    to_move = game.to_move
    to_move_string = cp.color_to_string(to_move)
    move_str = input(to_move_string + " to move:")

    while move_str not in ms.string_to_move[to_move]:
        print("Invalid move entered")
        move_str = input(to_move_string + " to move:")

    move = ms.parse_move(to_move, move_str)

    while move not in legal_moves:
        print("Illegal move attempted")
        # for n in range(0, len(legal_moves)):
        #     mv = legal_moves[n]
        #     print("legal move  " + str(n) + ": from: " + cp.file_to_letter(mv.fr[0]) + str(mv.fr[1]) + " to: " + cp.file_to_letter(mv.to[0]) + str(mv.to[1]) + "capture: " + str(mv.capture))

        for n in range(0, len(game.pieceMoveNums)):
            print("piece " + str(n) + " moved " + str(game.pieceMoveNums[n]) + " times")

        move_str = input(to_move_string + " to move:")

        while move_str not in ms.string_to_move[to_move]:
            print("Invalid move entered")
            move_str = input(to_move_string + " to move:")

        move = ms.parse_move(to_move, move_str)

    game.apply_move(move)

print(cp.state_to_string(gs.game_state(game)))

