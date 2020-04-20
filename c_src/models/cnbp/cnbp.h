#ifndef CNBP_H_INCLUDED
#define CNBP_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

/**
 * Categorical Naive Bayes with Partial Classification training data
 *
 * The features are categorical data (non numerical/continuous)
 *
 * The result of the classification is an array of probabilities between 0 and 1 whose total sum is 1 corresponding to
 * the probability of the provided data point of falling to the corresponding class.
 */

typedef struct categorical_naive_bayes_partial {

    /** The struct members are rearranged pointers first in order to guarantee packing **/

    /** Domain-specific parameters
     * categories       An array representing the number of different values in each category.
     * cat_num          Number of categories up to 255
     * class_num        Number of classes up to 255
     *
     * Numerical parameters
     * alpha            Value for laplacian smoothing
     *
     * Internals
     * total_cat_vals   Total number of different category values across all categories for indexing
     * class_cat_idx    Index for navigating the array for the per-class category totals
     * class_totals     Array of totals for classes
     * class_probs      Array of probabilities of classes
     * class_cat_totals Array of totals for each category per class
     * class_cat_probs  Array of probabilities of each category per class
     * is_fresh         1 if probabilities are fresh 0 if recalculation is needed.
     * use_probs        mode of operation 1 probabilities are used, 0 probabilities aren't used.
    **/
    uint8_t * categories;
    double * class_totals;
    double * class_probs;
    uint32_t * class_cat_idx;
    double * class_cat_totals;
    double * class_cat_probs;
    double alpha;
    uint32_t total_cat_vals;
    uint32_t cat_num;
    uint8_t class_num;
    uint8_t is_fresh;
    uint8_t use_probs;
} cnbp_t;

cnbp_t * create_cnbp_with_alpha(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        double alpha,
        uint8_t use_probs);

cnbp_t * create_cnbp(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        uint8_t use_probs);

void cnbp_train_single(
        cnbp_t * cnbp,
        const uint8_t * data,
        const double * label);

void cnbp_train_batch(
        cnbp_t * cnbp,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize);

void cnbp_train_batch_on_selected(
        cnbp_t * cnbp,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected);

void cnbp_predict_class(
        const cnbp_t * cnbp,
        const uint8_t * data,
        double * labels);

void cnbp_forget(
        cnbp_t * cnbp,
        double factor);

void free_cnbp(cnbp_t * cnbp);

cnbp_t * copy_cnbp(const cnbp_t * cnbp);

cnbp_t * cnbp_from_file_with_params(
        FILE * file,
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        double alpha,
        uint8_t use_probs);

cnbp_t * cnbp_from_file(
        FILE * file,
        uint8_t use_probs);

cnbp_t * cnbp_from_file_with_name(
        const char * filename,
        uint8_t use_probs);

void cnbp_to_file_no_params(
        const cnbp_t * cnbp,
        FILE * file);

void cnbp_to_file(
        const cnbp_t * cnbp,
        FILE * file);

void cnbp_to_file_with_name(
        const cnbp_t * cnbp,
        const char * filename);

#endif
