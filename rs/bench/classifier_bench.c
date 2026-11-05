/**
 * C counterpart of `chessai bench-cnbp`: trains the original cnbp.c on a games
 * directory, saves the model, predicts every training position and reports
 * throughput. Used to check the Rust port gives identical results and to
 * compare speed.
 *
 * Build: clang -O3 -o cnbp_bench cnbp_bench.c ../../c_src/models/cnbp/cnbp.c -lm
 * Usage: cnbp_bench GAMES_DIR MODEL_OUT PREDICTIONS_OUT ALPHA USE_PROBS REPS
 *
 * Training and prediction are each repeated REPS times and averaged.
 **/
#include "../../c_src/models/cnbp/cnbp.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define FEATURES 70

static const char * FILES[3] = {"white_wins.games", "black_wins.games", "draws.games"};

static double now(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec / 1e9;
}

int main(int argc, char ** argv)
{
    if (argc != 7) {
        fprintf(stderr, "usage: %s GAMES_DIR MODEL_OUT PREDICTIONS_OUT ALPHA USE_PROBS REPS\n", argv[0]);
        return 1;
    }

    double alpha = atof(argv[4]);
    uint8_t use_probs = (uint8_t) atoi(argv[5]);
    int reps = atoi(argv[6]);

    size_t cap = 1 << 16, n = 0;
    uint8_t (* data)[FEATURES] = malloc(cap * FEATURES);
    uint8_t * labels = malloc(cap);
    char line[1024], path[4096];

    for (int f = 0; f < 3; ++f) {
        snprintf(path, sizeof(path), "%s/%s", argv[1], FILES[f]);
        FILE * file = fopen(path, "r");
        if (!file) {
            fprintf(stderr, "cannot open %s\n", path);
            return 1;
        }
        while (fgets(line, sizeof(line), file)) {
            if (n == cap) {
                cap *= 2;
                data = realloc(data, cap * FEATURES);
                labels = realloc(labels, cap);
            }
            int at = 0;
            for (char * tok = strtok(line, " \n"); tok && at < FEATURES; tok = strtok(NULL, " \n")) {
                data[n][at++] = (uint8_t) atoi(tok);
            }
            if (at == FEATURES) {
                labels[n++] = (uint8_t) f;
            }
        }
        fclose(file);
    }

    uint8_t cats[FEATURES];
    for (int i = 0; i < 64; ++i) cats[i] = 13;
    cats[64] = 2; cats[65] = 9; cats[66] = cats[67] = cats[68] = cats[69] = 2;

    cnbp_t * model = NULL;
    double onehot[3][3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
    double out[3];

    double start = now();
    for (int r = 0; r < reps; ++r) {
        free_cnbp(model);
        model = create_cnbp_with_alpha(3, cats, FEATURES, alpha, use_probs);
        for (size_t i = 0; i < n; ++i) {
            cnbp_train_single(model, data[i], onehot[labels[i]]);
        }
        /* cnbp recalculates cached probabilities lazily on the first
         * prediction; count that as part of training, as Rust does. */
        cnbp_predict_class(model, data[0], out);
    }
    double train_secs = (now() - start) / reps;

    cnbp_to_file_with_name(model, argv[2]);

    double (* preds)[3] = malloc(n * sizeof(* preds));
    start = now();
    for (int r = 0; r < reps; ++r) {
        for (size_t i = 0; i < n; ++i) {
            cnbp_predict_class(model, data[i], preds[i]);
        }
    }
    double predict_secs = (now() - start) / reps;

    FILE * pf = fopen(argv[3], "w");
    for (size_t i = 0; i < n; ++i) {
        fprintf(pf, "%.17g %.17g %.17g\n", preds[i][0], preds[i][1], preds[i][2]);
    }
    fclose(pf);

    printf("c    cnbp: %zu positions, train %.1fM pos/s, predict %.2fM pos/s\n",
           n, n / train_secs / 1e6, n / predict_secs / 1e6);

    free_cnbp(model);
    free(preds);
    free(data);
    free(labels);
    return 0;
}
