from collections import deque
import chess_consts as cc

initialPieces = [
    (cc.White, cc.Rook),
    (cc.White, cc.Knight),
    (cc.White, cc.Bishop),
    (cc.White, cc.Queen),
    (cc.White, cc.King),
    (cc.White, cc.Bishop),
    (cc.White, cc.Knight),
    (cc.White, cc.Pawn),
    (cc.White, cc.Pawn),
    (cc.White, cc.Pawn),
    (cc.White, cc.Pawn),
    (cc.White, cc.Pawn),
    (cc.White, cc.Pawn),
    (cc.White, cc.Pawn),
    (cc.White, cc.Pawn),
    (cc.Black, cc.Rook),
    (cc.Black, cc.Knight),
    (cc.Black, cc.Bishop),
    (cc.Black, cc.Queen),
    (cc.Black, cc.King),
    (cc.Black, cc.Bishop),
    (cc.Black, cc.Knight),
    (cc.Black, cc.Pawn),
    (cc.Black, cc.Pawn),
    (cc.Black, cc.Pawn),
    (cc.Black, cc.Pawn),
    (cc.Black, cc.Pawn),
    (cc.Black, cc.Pawn),
    (cc.Black, cc.Pawn),
    (cc.Black, cc.Pawn)
]

initialPositions = [
    deque([(cc.A, 1)]),
    deque([(cc.B, 1)]),
    deque([(cc.C, 1)]),
    deque([(cc.D, 1)]),
    deque([(cc.E, 1)]),
    deque([(cc.F, 1)]),
    deque([(cc.G, 1)]),
    deque([(cc.H, 1)]),
    deque([(cc.A, 2)]),
    deque([(cc.B, 2)]),
    deque([(cc.C, 2)]),
    deque([(cc.D, 2)]),
    deque([(cc.E, 2)]),
    deque([(cc.F, 2)]),
    deque([(cc.G, 2)]),
    deque([(cc.H, 2)]),
    deque([(cc.A, 8)]),
    deque([(cc.B, 8)]),
    deque([(cc.C, 8)]),
    deque([(cc.D, 8)]),
    deque([(cc.E, 8)]),
    deque([(cc.F, 8)]),
    deque([(cc.G, 8)]),
    deque([(cc.H, 8)]),
    deque([(cc.A, 7)]),
    deque([(cc.B, 7)]),
    deque([(cc.C, 7)]),
    deque([(cc.D, 7)]),
    deque([(cc.E, 7)]),
    deque([(cc.F, 7)]),
    deque([(cc.G, 7)]),
    deque([(cc.H, 7)])
]

initialBoard = [
    [deque([0]), deque([1]), deque([2]), deque([3]), deque([4]), deque([5]), deque([6]), deque([7])],
    [deque([8]), deque([9]), deque([10]), deque([11]), deque([12]), deque([13]), deque([14]), deque([15])],
    [deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None])],
    [deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None])],
    [deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None])],
    [deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None]), deque([None])],
    [deque([24]), deque([25]), deque([26]), deque([27]), deque([28]), deque([29]), deque([30]), deque([31])],
    [deque([16]), deque([17]), deque([18]), deque([19]), deque([20]), deque([21]), deque([22]), deque([23])]
]

initialPieceMoveNums = [
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0
]

