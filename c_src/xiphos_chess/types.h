#ifndef XIPHOS_TYPES_H
#define XIPHOS_TYPES_H

#include "game.h"

#define MAX_GAME_PLY  1024

typedef uint32_t move_t;

typedef struct {
    uint64_t phash_key, // 8
            occ[N_SIDES], // 16
            piece_occ[N_PIECES - 1], // 40
            pinned[N_SIDES], //16
            pinners[N_SIDES]; //16
    move_t move; //4
    uint8_t board[BOARD_SIZE]; //64
    uint8_t side, //1
            ep_sq, //1
            c_flag, //1
            in_check, //1
            fifty_cnt, //1
            k_sq[N_SIDES]; //2
} __attribute__ ((aligned (16))) position_t;
_Static_assert(sizeof(position_t) == 176, "position_t size error");

typedef struct {
    int tid, hash_keys_cnt;
    uint64_t nodes, tbhits, hash_key;
    position_t *pos,
            pos_list[PLY_LIMIT];
    move_t   killer_moves[PLY_LIMIT][MAX_KILLER_MOVES],
            counter_moves[P_LIMIT][BOARD_SIZE];
    int16_t  history[N_SIDES][BOARD_SIZE][BOARD_SIZE],
            counter_move_history[P_LIMIT][BOARD_SIZE][P_LIMIT * BOARD_SIZE];
    uint64_t hash_keys[MAX_GAME_PLY];
} search_data_t;

#endif
