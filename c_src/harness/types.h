#ifndef CHESSAI_HARNESS_TYPES_H
#define CHESSAI_HARNESS_TYPES_H

typedef void * (*new_model_t)(const char **);
typedef void * (*read_model_t)(const char *, const char **);
typedef void (*write_model_t)(const void *, const char *);

typedef void (*train_model_t)(void *, const uint8_t *, const double *);
typedef void (*run_model_t)(const void *, const uint8_t *, double *);
typedef void (*free_model_t)(void *);

typedef struct evaluation_model {
    void * model;
    new_model_t new_model;
    read_model_t read_model;
    write_model_t write_model;
    train_model_t train_model;
    run_model_t run_model;
    free_model_t free_model;
} em_t;

typedef void * (*select_move_t)(const uint8_t * const *, size_t, double *);

typedef struct player {
    em_t * eval_model;
    select_move_t select_move;
} player_t;

#endif //C_SRC_TYPES_H
