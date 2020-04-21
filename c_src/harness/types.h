#ifndef CHESSAI_HARNESS_TYPES_H
#define CHESSAI_HARNESS_TYPES_H

#include <stdint.h>

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

typedef enum player_color {
    WHITE = 0,
    BLACK = 1
} color_t;

typedef int (*compare_moves_t)(color_t, const double *, const double *);

typedef struct player {
    em_t * eval_model;
    compare_moves_t compare_moves;
    color_t color;
} player_t;

#endif //C_SRC_TYPES_H
