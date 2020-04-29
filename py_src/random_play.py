import sys
import os
import random
import harness.model as modellib
import harness.player as pl
import chess.categorical_input as ci
import chess.chess_game as cg
import chess.chess_consts as cc
import chess.game_state as gs
import chess.moves as moves
import chess.move_strings as ms

round_num = int(sys.argv[1])
game_num = int(sys.argv[2])
model_type = sys.argv[3]
randomplay_dir = sys.argv[4]
model_version = int(sys.argv[5])
if model_version == 0:
    model_args = [3] + [70] + ci.game_classes() + sys.argv[6].split(",")
else:
    model_args = sys.argv[6].split(",")
player_move_compare = sys.argv[7]

print("Loading Player model")
print("Model type: " + model_type)
print("Model version: " + str(model_version))
print("Selfplay directory: " + randomplay_dir)
print("Model args: " + str(model_args))

model = modellib.model_c.load_model(
    model_type,
    model_version,
    randomplay_dir,
    model_args)

print("Creating Player")
print("Player move comparison type: " + player_move_compare)
print("Player starting color: White")

player = pl.player_c.player(
    model,
    player_move_compare,
    cc.White)


resultsFile = open(randomplay_dir + "/results.txt", "a")
at_round = model_version

while at_round < round_num:

    game_dir = os.path.join(randomplay_dir, "round" + str(at_round))
    os.makedirs(game_dir)

    player_win_num = 0
    random_win_num = 0
    draw_num = 0

    whitewins = open(game_dir + "/white_wins.games", "w")
    blackwins = open(game_dir + "/black_wins.games", "w")
    draws = open(game_dir + "/draws.games", "w")
    human = open(game_dir + "/human.games", "w")

    player_side = cc.White
    pl.player_c.set_color(player, cc.White)
    print("Beginning round " + str(at_round))

    for n in range(0, game_num):
        game = cg.ChessGame()
        move_num = 0
        position_inputs = []
        human_game = ""

        while gs.game_state(game) == cc.InProgress:
            to_move = game.to_move

            all_moves = moves.all_legal_moves(game, ci.game_as_input)
            available_moves = all_moves[0]
            resulting_positions = all_moves[1]

            if to_move == player_side:
                idx = pl.player_c.select_move(
                    player,
                    resulting_positions)
            else:
                idx = random.randrange(0, len(resulting_positions))

            if to_move == cc.White:
                move_num += 1
                human_game += (str(move_num) + ". ")
                to_move = cc.Black
            else:
                to_move = cc.White

            pos = (str(resulting_positions[idx])[1:-1])
            pos = pos.replace(',', '') + '\n'
            position_inputs.append(pos)

            human_game += ms.move_to_string(available_moves[idx], game) + " "
            game.apply_move(available_moves[idx])

        pos = (str(ci.game_as_input(game))[1:-1])
        pos = pos.replace(',', '') + '\n'
        position_inputs.append(pos)
        state = gs.game_state(game)

        printstr = "Round: " + str(at_round) + " Game: " + str(n) + " Result: "

        if state == cc.Draw:
            human_game += "1/2-1/2\n\n"
            for p in position_inputs:
                draws.write(p)
        elif state == cc.WhiteWin:
            for p in position_inputs:
                whitewins.write(p)
            human_game += "1-0\n\n"
        else:
            for p in position_inputs:
                blackwins.write(p)
            human_game += "0-1\n\n"

        if state == player_side:
            printstr += "player wins"
            player_win_num += 1
        elif state == cc.Draw:
            printstr += "draw"
            draw_num += 1
        else:
            printstr += "random wins"
            random_win_num += 1
        print(printstr)
        human.write(human_game)

        pl.player_c.switch_color(player)

        if player_side == cc.White:
            player_side = cc.Black
        else:
            player_side = cc.White

    draws.close()
    whitewins.close()
    blackwins.close()
    human.close()

    resultsFile.write("round " + str(at_round) + " player score: +" + str(player_win_num) + "-" + str(random_win_num) + "=" + str(draw_num) + "\n")
    resultsFile.flush()

    print("Training model on games of round " + str(at_round))
    modellib.model_c.train_model(
        model,
        game_dir
    )

    model_version += 1
    modellib.model_c.store_model(
        model,
        model_type,
        model_version,
        randomplay_dir)

    pl.player_c.free_player(player)

    model = modellib.model_c.load_model(
        model_type,
        model_version,
        randomplay_dir,
        model_args)

    player = pl.player_c.player(
        model,
        player_move_compare,
        cc.White)

    at_round += 1

resultsFile.close()
