import sys
import harness.model as model
import harness.player as player
import chess.categorical_input as ci
import chess.chess_consts as cc
import chess.chess_game as cg
import chess.game_state as gs
import chess.move_strings as ms
import chess.moves as moves


game_num = int(sys.argv[1])
game_dir = sys.argv[2]

model1_type = sys.argv[3]
model1_dir = sys.argv[4]
model1_version = int(sys.argv[5])
model1_args = [3] + [70] + ci.game_classes() + sys.argv[6].split(",")
player1_move_compare = sys.argv[7]

model2_type = sys.argv[8]
model2_dir = sys.argv[9]
model2_version = int(sys.argv[10])
model2_args = [3] + [70] + ci.game_classes() + sys.argv[11].split(",")
player2_move_compare = sys.argv[12]

print("Loading Player 1 model")
print("Model 1 type: " + model1_type)
print("Model 1 version: " + str(model1_version))
print("Model 1 directory: " + model1_dir)
print("Model 1 args: " + str(model1_args))

model1 = model.model_c.load_model(
    model1_type,
    model1_version,
    model1_dir,
    model1_args)

print("Creating Player 1")
print("Player 1 move comparison type: " + player1_move_compare)
print("Player 1 starting color: White")

player1 = player.player_c.player(
    model1,
    player1_move_compare,
    cc.White)
player1_side = cc.White

print("Loading Player 2 model")
print("Model 2 type: " + model2_type)
print("Model 2 version: " + str(model2_version))
print("Model 2 directory: " + model2_dir)
print("Model 2 args: " + str(model2_args))

model2 = model.model_c.load_model(
    model2_type,
    model2_version,
    model2_dir,
    model2_args)

print("Creating Player 2")
print("Player 2 move comparison type: " + player2_move_compare)
print("Player 2 starting color: Black")

player2 = player.player_c.player(
    model2,
    player2_move_compare,
    cc.Black)

print("Games to be played: " + str(game_num))
print("Games stored in directory: " + game_dir)

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

        all_moves = moves.all_legal_moves(game, ci.game_as_input)
        available_moves = all_moves[0]
        resulting_positions = all_moves[1]
        idx = 0

        if to_move == player1_side:
            idx = player.player_c.select_move(
                player1,
                resulting_positions)

        else:
            idx = player.player_c.select_move(
                player2,
                resulting_positions)

        pos = (str(all_moves[1][idx])[1:-1])
        pos = pos.replace(',', '') + '\n'
        position_inputs.append(pos)
        human_game += ms.move_to_string(available_moves[idx], game)
        human_game += " "
        game.apply_move(available_moves[idx])

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

