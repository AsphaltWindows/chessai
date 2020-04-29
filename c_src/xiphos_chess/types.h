#ifndef XIPHOS_TYPES_H
#define XIPHOS_TYPES_H

typedef uint32_t move_t;

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
