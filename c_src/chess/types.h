#ifndef CHESS_TYPES_H
#define CHESS_TYPES_H

#define UINT32_BOARD_SIZE 10
#define FLAGS_INDEX 8
#define GARBAGE_INDEX 9
#define SQUARE_NUM 64

typedef enum FILE {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7
} file_t;

typedef enum RANK {
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7
} rank_t;

typedef enum SQUARE {
    A1 = 0, A2 = 1, A3 = 2, A4 = 3, A5 = 4, A6 = 5, A7 = 6, A8 = 7,
    B1 = 8, B2 = 9, B3 = 10, B4 = 11, B5 = 12, B6 = 13, B7 = 14, B8 = 15,
    C1 = 16, C2 = 17, C3 = 18, C4 = 19, C5 = 20, C6 = 21, C7 = 22, C8 = 23,
    D1 = 24, D2 = 25, D3 = 26, D4 = 27, D5 = 28, D6 = 29, D7 = 30, D8 = 31,
    E1 = 32, E2 = 33, E3 = 34, E4 = 35, E5 = 36, E6 = 37, E7 = 38, E8 = 39,
    F1 = 40, F2 = 41, F3 = 42, F4 = 43, F5 = 44, F6 = 45, F7 = 46, F8 = 47,
    G1 = 48, G2 = 49, G3 = 50, G4 = 51, G5 = 52, G6 = 53, G7 = 54, G8 = 55,
    H1 = 56, H2 = 57, H3 = 58, H4 = 59, H5 = 60, H6 = 61, H7 = 62, H8 = 63
} square_t;

typedef enum COLOR {
    WHITE = 0,
    BLACK = 1
} color_t;

typedef enum PIECETYPE {
    PAWN = 1,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    KING = 6
} piecetype_t;

typedef enum PIECE {
    NONE = 0,
    WHITE_PAWN = 1,
    WHITE_KNIGHT = 2,
    WHITE_BISHOP = 3,
    WHITE_ROOK = 4,
    WHITE_QUEEN = 5,
    WHITE_KING = 6,
    BLACK_PAWN = 9,
    BLACK_KNIGHT = 10,
    BLACK_BISHOP = 11,
    BLACK_ROOK = 12,
    BLACK_QUEEN = 13,
    BLACK_KING = 14,
    PIECE_T_NUM = 15
} piece_t;

typedef enum CASTLE {
    NOCASTLE = 0,
    WHITELONGCASTLE = 1,
    WHITESHORTCASTLE = 2,
    BLACKLONGCASTLE = 5,
    BLACKSHORTCASTLE = 6
} castle_t;

typedef enum ENPASSANT {
    NO_ENPASSANT = 0,
    ENPASSANT = 1
} enpassant_t;

typedef uint32_t move_t;

typedef struct chess_position {
    uint32_t bb[UINT32_BOARD_SIZE];
} position_t;

typedef int g_state_t;

#endif //CHESS_TYPES_H
