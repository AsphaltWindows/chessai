/**
 * C counterpart of `chessai bench-cnbp` and `chessai bench-bdt`: trains the
 * original cnbp.c or bdt.c on a games directory, saves the model, predicts
 * every training position and reports throughput. Used to check the Rust
 * ports give identical results and to compare speed.
 *
 * Build: clang -O3 -o classifier_bench classifier_bench.c \
 *            ../../c_src/models/cnbp/cnbp.c ../../c_src/models/bdt/bdt.c -lm
 * Usage: classifier_bench cnbp GAMES_DIR MODEL_OUT PREDICTIONS_OUT REPS ALPHA USE_PROBS
 *        classifier_bench bdt  GAMES_DIR MODEL_OUT PREDICTIONS_OUT REPS ALPHA USE_PROBS
 *                              BRANCH_FACTOR SPLIT_THRESHOLD SPLIT_LIMIT FORGET_FACTOR
 *
 * Training and prediction are each repeated REPS times and averaged.
 **/
#include "../../c_src/models/cnbp/cnbp.h"
#include "../../c_src/models/bdt/bdt.h"

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

typedef struct {
    void * (* create)(char ** argv, uint8_t * cats);
    void (* train)(void * model, const uint8_t * data, const double * label);
    void (* predict)(void * model, const uint8_t * data, double * out);
    void (* save)(void * model, const char * path);
    void (* destroy)(void * model);
} classifier_t;

static void * cnbp_create(char ** argv, uint8_t * cats)
{
    return create_cnbp_with_alpha(3, cats, FEATURES, atof(argv[6]), (uint8_t) atoi(argv[7]));
}
static void cnbp_train(void * m, const uint8_t * d, const double * l) { cnbp_train_single(m, d, l); }
static void cnbp_predict(void * m, const uint8_t * d, double * o) { cnbp_predict_class(m, d, o); }
static void cnbp_save(void * m, const char * p) { cnbp_to_file_with_name(m, p); }
static void cnbp_destroy(void * m) { free_cnbp(m); }

static void * bdt_create(char ** argv, uint8_t * cats)
{
    return create_bdt(cats, FEATURES, 3, (uint8_t) atoi(argv[8]), (uint32_t) atoi(argv[9]),
                      (uint32_t) atoi(argv[10]), atof(argv[11]), atof(argv[6]), (uint8_t) atoi(argv[7]));
}
static void bdt_train(void * m, const uint8_t * d, const double * l) { bdt_train_single(m, d, l); }
static void bdt_predict(void * m, const uint8_t * d, double * o) { bdt_predict_class(m, d, o); }
static void bdt_save(void * m, const char * p) { bdt_to_file_with_name(m, p); }
static void bdt_destroy(void * m) { free_bdt(m); }

int main(int argc, char ** argv)
{
    classifier_t c;

    if (argc == 8 && !strcmp(argv[1], "cnbp")) {
        c = (classifier_t) {cnbp_create, cnbp_train, cnbp_predict, cnbp_save, cnbp_destroy};
    } else if (argc == 12 && !strcmp(argv[1], "bdt")) {
        c = (classifier_t) {bdt_create, bdt_train, bdt_predict, bdt_save, bdt_destroy};
    } else {
        fprintf(stderr, "usage: %s cnbp GAMES_DIR MODEL_OUT PREDICTIONS_OUT REPS ALPHA USE_PROBS\n"
                        "       %s bdt  GAMES_DIR MODEL_OUT PREDICTIONS_OUT REPS ALPHA USE_PROBS "
                        "BRANCH_FACTOR SPLIT_THRESHOLD SPLIT_LIMIT FORGET_FACTOR\n", argv[0], argv[0]);
        return 1;
    }

    const char * games_dir = argv[2];
    int reps = atoi(argv[5]);

    size_t cap = 1 << 16, n = 0;
    uint8_t (* data)[FEATURES] = malloc(cap * FEATURES);
    uint8_t * labels = malloc(cap);
    char line[1024], path[4096];

    for (int f = 0; f < 3; ++f) {
        snprintf(path, sizeof(path), "%s/%s", games_dir, FILES[f]);
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

    void * model = NULL;
    double onehot[3][3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
    double out[3];

    double start = now();
    for (int r = 0; r < reps; ++r) {
        if (model) {
            c.destroy(model);
        }
        model = c.create(argv, cats);
        for (size_t i = 0; i < n; ++i) {
            c.train(model, data[i], onehot[labels[i]]);
        }
        /* With USE_PROBS the C models recalculate cached probabilities lazily
         * on the next prediction; count that as training, as Rust does. */
        c.predict(model, data[0], out);
    }
    double train_secs = (now() - start) / reps;

    c.save(model, argv[3]);

    double (* preds)[3] = malloc(n * sizeof(* preds));
    start = now();
    for (int r = 0; r < reps; ++r) {
        for (size_t i = 0; i < n; ++i) {
            c.predict(model, data[i], preds[i]);
        }
    }
    double predict_secs = (now() - start) / reps;

    FILE * pf = fopen(argv[4], "w");
    for (size_t i = 0; i < n; ++i) {
        fprintf(pf, "%.17g %.17g %.17g\n", preds[i][0], preds[i][1], preds[i][2]);
    }
    fclose(pf);

    printf("c    %s: %zu positions, train %.2fM pos/s, predict %.2fM pos/s\n",
           argv[1], n, n / train_secs / 1e6, n / predict_secs / 1e6);

    c.destroy(model);
    free(preds);
    free(data);
    free(labels);
    return 0;
}
