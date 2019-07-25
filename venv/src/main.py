import chess.chess_game as cg
import chess.chess_parse as cp
import chess.game_state as gs
import chess.chess_consts as cc
import chess.move_strings as ms
import chess.moves as m


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

