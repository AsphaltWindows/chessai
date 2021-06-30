#ifndef CNBP_SC_H_INCLUDED
#define CNBP_SC_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "../cnbp/cnbp.h"


/**
 * CNBP Scatter Chain
 *
 *** Structure ***
 *
 * This classification model is a chain of a static size
 *
 * It is the same as CNBP CHAIN, except during a node in the chain will be selected
 * randomly and only that node will be trained by the data point.
 *
 */

typedef struct cnbp_sc {
    cnbp_t ** nodes;
    uint8_t * categories;
    size_t cat_num;
    size_t nodes_num;
    double nb_alpha;
    uint8_t class_num;
    uint8_t use_probs;
} cnbp_sc_t;

cnbp_sc_t * create_cnbp_sc(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        size_t nodes_num,
        double nb_alpha,
        uint8_t use_probs);

void cnbp_sc_train_single(
        cnbp_sc_t * cnbp_sc,
        const uint8_t * data,
        const double * label);

void cnbp_sc_train_batch(
        cnbp_sc_t * cnbp_sc,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize);

void cnbp_sc_predict_class(
        const cnbp_sc_t * cnbp_sc,
        const uint8_t * data,
        double * labels);

void free_cnbp_sc(
        cnbp_sc_t * cnbp_sc);

cnbp_sc_t * cnbp_sc_from_file(
        FILE * file,
        uint8_t use_probs);

cnbp_sc_t * cnbp_sc_from_file_with_name(
        const char * filename,
        uint8_t use_probs);

void cnbp_sc_to_file(
        const cnbp_sc_t * cnbp_sc,
        FILE * file);

void cnbp_sc_to_file_with_name(
        const cnbp_sc_t * cnbp_sc,
        const char * filename);

#endif
