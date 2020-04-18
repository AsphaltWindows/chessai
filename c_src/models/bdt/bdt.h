#ifndef BDT_H_INCLUDED
#define BDT_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "../cnbp/cnbp.h"

/**
 * Bayesian Delegation Tree Classification model
 *
 *** Structure ***
 *
 * This classification model is a dynamically expanding tree,
 * which consists of leaf and branch nodes.
 *
 * Leaf Nodes are classifiers for the target problem but for only a subset of the inputs.
 *
 * Branch Nodes are classifiers that predict which of their children subtrees
 * would be best suited for classifying a given input.
 *
 *** Classification ***
 *
 * During classification each branch node starting with the root attempts to
 * predict which of its children will provide the most accurate result and calls for that
 * child node to classify the data.
 *
 * When a leaf node is reached by these calls its result is
 * the result of the whole.
 *
 *** Training ***
 *
 * During training each branch node retrieves a classification for the input from each of its
 * children nodes, computes the error for each child, updates itself with that child node as
 * the correct label for the data and class on that child node to train itself on the data.
 * When a leaf node is reached by these calls it should have been passed along the desired
 * label for that data from the original training call and it trains itself accordingly.
 *
 *** Dynamic Expansion ***
 * Using a decision procedure local to the node, a leaf node can split into several leaf
 * nodes and a branch node, which remains the child of the leaf's parent. The split creates
 * leafs equal to the branch_factor which are identical in predictive behavior to the
 * original leaf, but may be modified to accelerate their future training. In the case
 * of the Naive Bayes Classifier nodes this can be done by scaling down the frequencies.
 */

typedef enum node_type {
    LEAF = 0,
    BRANCH = 1
} ntype_t;

typedef struct bdt_node bdt_node_t;
typedef struct bayesian_delegation_tree bdt_t;

struct bdt_node {
    /** The struct members are rearranged pointers first in order to guarantee packing **/
    bdt_t * root_bdt;
    cnbp_t * classifier;
    uint32_t * children;
    ntype_t type;
    uint32_t num_trained;
};

struct bayesian_delegation_tree {
    /** The struct members are rearranged pointers first in order to guarantee packing **/
    uint8_t * categories;
    bdt_node_t ** nodes;
    uint8_t cat_num;
    size_t nodes_num;
    uint8_t class_num;
    uint8_t branch_factor;
    uint32_t split_threshold;
    uint32_t split_limit;
    uint32_t split_number;
    double forget_factor;
    double nb_alpha;
};

bdt_t * create_bdt(
        uint8_t * categories,
        uint8_t cat_num,
        uint8_t class_num,
        uint8_t branch_factor,
        uint32_t split_threshold,
        uint32_t split_limit,
        double forget_factor,
        double nb_alpha);

void train_single(
        bdt_t * bdt,
        const uint8_t * data,
        const double * label);

void train_batch(
        bdt_t * bdt,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize);

double * predict_class(
        const bdt_t * bdt,
        const uint8_t * data);

void free_bdt(
        bdt_t * bdt);

bdt_t * bdt_from_file(
        FILE * file);

void bdt_to_file(
        const bdt_t * bdt,
        FILE * file);


#endif
