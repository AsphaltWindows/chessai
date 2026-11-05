"""Play seeded random games with the Python move generator and dump them.

Writes, into OUT_DIR:
  xcheck.txt        one line per position: legal moves (UCI, sorted) | encoding | move played
                    and a final "end <result>" line per game
  white_wins.games  positions in the same format self_play.py writes,
  black_wins.games  so they can be used as CNBP training data
  draws.games

Usage: python3 rs/bench/dump_python_games.py GAMES SEED OUT_DIR
"""
import os
import random
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "py_src"))

import chess.categorical_input as ci
import chess.chess_consts as cc
import chess.chess_game as cg
import chess.game_state as gs
import chess.moves as moves

FILES = "abcdefgh"
PROMOTIONS = {cc.Knight: "n", cc.Bishop: "b", cc.Rook: "r", cc.Queen: "q"}
RESULTS = {cc.WhiteWin: "1-0", cc.BlackWin: "0-1", cc.Draw: "1/2-1/2"}


def to_uci(move):
    if move.longCastleWhite:
        return "e1c1"
    if move.shortCastleWhite:
        return "e1g1"
    if move.longCastleBlack:
        return "e8c8"
    if move.shortCastleBlack:
        return "e8g8"
    uci = FILES[move.fr[0] - 1] + str(move.fr[1]) + FILES[move.to[0] - 1] + str(move.to[1])
    if move.promote is not None:
        uci += PROMOTIONS[move.promote]
    return uci


def as_line(position):
    return " ".join(str(v) for v in position) + "\n"


game_num = int(sys.argv[1])
random.seed(int(sys.argv[2]))
out_dir = sys.argv[3]
os.makedirs(out_dir, exist_ok=True)

outputs = {
    cc.WhiteWin: open(os.path.join(out_dir, "white_wins.games"), "w"),
    cc.BlackWin: open(os.path.join(out_dir, "black_wins.games"), "w"),
    cc.Draw: open(os.path.join(out_dir, "draws.games"), "w"),
}
xcheck = open(os.path.join(out_dir, "xcheck.txt"), "w")

plies = 0
start = time.time()

for n in range(game_num):
    game = cg.ChessGame()
    positions = []

    while gs.game_state(game) == cc.InProgress:
        available_moves, resulting_positions = moves.all_legal_moves(game, ci.game_as_input)
        idx = random.randrange(len(available_moves))
        legal = " ".join(sorted(to_uci(m) for m in available_moves))
        xcheck.write(legal + " | " + as_line(ci.game_as_input(game)).strip() + " | " + to_uci(available_moves[idx]) + "\n")
        positions.append(as_line(resulting_positions[idx]))
        game.apply_move(available_moves[idx])
        plies += 1

    positions.append(as_line(ci.game_as_input(game)))
    state = gs.game_state(game)
    xcheck.write("end " + RESULTS[state] + "\n")
    outputs[state].writelines(positions)

elapsed = time.time() - start
print(f"{game_num} games, {plies} plies in {elapsed:.1f}s ({plies / elapsed:.0f} plies/s)")

for f in list(outputs.values()) + [xcheck]:
    f.close()
