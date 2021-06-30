#ifndef CNBP_CHAIN_H_INCLUDED
#define CNBP_CHAIN_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "../cnbp/cnbp.h"


/**
 * CNBP Chain
 *
 *** Structure ***
 *
 * This classification model is a chain of a static size
 *
 * Each node in the chain is a CNBP classifer, which receives all the same parameters as
 * the previous CNBP node, but in addition the most likely classification estimate of the previous node.
 *
 * This means that at every level in the chain the number of categories inputted into the CNBP at that level increases
 * by 1 with an additional category the size of which is equal to the number of classes.
 */

typedef struct cnbp_chain {
    cnbp_t ** nodes;
    uint8_t * categories;
    size_t cat_num;
    size_t nodes_num;
    double nb_alpha;
    uint8_t class_num;
    uint8_t use_probs;
} cnbp_chain_t;

cnbp_chain_t * create_cnbp_chain(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        size_t nodes_num,
        double nb_alpha,
        uint8_t use_probs);

void cnbp_chain_train_single(
        cnbp_chain_t * cnbp_chain,
        const uint8_t * data,
        const double * label);

void cnbp_chain_train_batch(
        cnbp_chain_t * cnbp_chain,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize);

void cnbp_chain_predict_class(
        const cnbp_chain_t * cnbp_chain,
        const uint8_t * data,
        double * labels);

void free_cnbp_chain(
        cnbp_chain_t * cnbp_chain);

cnbp_chain_t * cnbp_chain_from_file(
        FILE * file,
        uint8_t use_probs);

cnbp_chain_t * cnbp_chain_from_file_with_name(
        const char * filename,
        uint8_t use_probs);

void cnbp_chain_to_file(
        const cnbp_chain_t * cnbp_chain,
        FILE * file);

void cnbp_chain_to_file_with_name(
        const cnbp_chain_t * cnbp_chain,
        const char * filename);

#endif
