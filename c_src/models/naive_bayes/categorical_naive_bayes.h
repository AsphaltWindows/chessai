#ifndef K_CNB_H_INCLUDED
#define K_CNB_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

typedef struct categorical_naive_bayes {
    uint8_t class_num;
    double alpha;
    uint8_t * categories;
    uint8_t cat_num;
    uint32_t * class_totals;
    double * class_probs;
    uint32_t total_cat_vals;
    uint32_t * class_cat_idx;
    uint32_t * class_cat_totals;
    double * class_cat_probs;
} cnb_clas_t;

void train_batch(
        cnb_clas_t *cnb,
        const uint8_t * const * data,
        const uint8_t * labels,
        size_t dsize);

void train_batch_on_selected(
        cnb_clas_t *cnb,
        const uint8_t * const * data,
        const uint8_t * labels,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected);

double * predict_class(
        const cnb_clas_t * cnb,
        const uint8_t * data);

cnb_clas_t * create_cnb_with_alpha(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        double alpha);

cnb_clas_t * create_cnb(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num);

cnb_clas_t * cnb_model_from_vals(
        const uint32_t * values,
        size_t num_values);

uint32_t * cnb_model_to_vals(
        const cnb_clas_t * cnb);

cnb_clas_t * cnb_from_file(
        FILE * file);

void cnb_to_file(
        const cnb_clas_t * cnb,
        FILE * file);

void recalculate_probabilities(
        cnb_clas_t *cnb);

void free_cnb(cnb_clas_t *cnb);

#endif
