#ifndef CHESSAI_HARNESS_MODEL_H
#define CHESSAI_HARNESS_MODEL_H

#include "types.h"

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#define RANDOM  "rand"
#define NB      "nb"
#define BDT     "bdt"

em_t * model(
        const char * model_type);

void new_model(
        em_t * model,
        const char ** args);

void free_model(em_t * model);

em_t * load_model(
        const char * model_type,
        uint32_t model_version,
        const char * directory,
        const char ** args);

void store_model(
        em_t * model,
        const char * model_type,
        uint32_t model_version,
        const char * directory);

void train_model(
        em_t * model,
        const char * directory);

#endif
