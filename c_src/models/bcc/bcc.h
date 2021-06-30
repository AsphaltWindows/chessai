#ifndef BCC_H_INCLUDED
#define BCC_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "../cnbp/cnbp.h"


/**
 * Bayesian Clarification Complex Classification model
 *
 *** Structure ***
 *
 * This classification model is a static tree 1 level deep. There is a root node and class_num
 * children nodes.
 *
 *** Classification ***
 *
 * The root node performs preliminary classification via a CNBP. It then sends the data off to one
 * of class_num children, depending on the what the initial node believed the accurate classification was.
 * That node then performs the classification and this enter is propagated as the result.
 *
 *** Training ***
 *
 * During the training the root node is first trained. Then, the root node is used to predict the class
 * based on the data and the appropriate children node is then trained.
 *
 */

typedef struct bayesian_clarification_complex {
    /** The struct members are rearranged pointers first in order to guarantee packing **/
    uint8_t * categories;
    cnbp_t * root;
    cnbp_t ** children;
    uint8_t class_num;
    size_t cat_num;
    double nb_alpha;
    uint8_t use_probs;
} bcc_t;

bcc_t * create_bcc(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        double nb_alpha,
        uint8_t use_probs);

void bcc_train_single(
        bcc_t * bcc,
        const uint8_t * data,
        const double * label);

void bcc_train_batch(
        bcc_t * bcc,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize);

void bcc_predict_class(
        const bcc_t * bcc,
        const uint8_t * data,
        double * labels);

void free_bcc(
        bcc_t * bcc);

bcc_t * bcc_from_file(
        FILE * file,
        uint8_t use_probs);

bcc_t * bcc_from_file_with_name(
        const char * filename,
        uint8_t use_probs);

void bcc_to_file(
        const bcc_t * bcc,
        FILE * file);

void bcc_to_file_with_name(
        const bcc_t * bcc,
        const char * filename);

#endif
