#ifndef BDT_H_INCLUDED
#define BDT_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "../cnbp/cnbp.h"

#define BRANCH_NUM 2

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
 * During classification the root node attempts to classify the data.
 * If the root node is a branch node it classifies the data to predict which of its children
 * would perform best at classifying this data and calls on that node to classify recursively.
 *
 * When a leaf node is reached by these calls it classifies the data using its classifier
 * and this result is passed up the call stack.
 *
 *** Training ***
 *
 * During the training the root nodes attempts to train first.
 * If the root node is a branch node it first calls for all of its children to classify
 * the data while withholding the label. It then calculates the error each of its children
 * generated in comparison to the label. The branch node then trains its own classifier
 * with the correct label for the data being the index of the child that had the least error.
 * It then recursively calls on only that child to train on the original data and label.
 *
 *** Dynamic Expansion ***
 * Using a decision procedure local to the node, a leaf node can split into several leaf
 * nodes and a branch node, which remains the child of the original leaf's parent.
 * The split creates BRANCH_NUM(2) new leafs which are identical in predictive
 * behavior to the original leaf, but may be modified to accelerate their future training.
 * In the case of the Naive Bayes Classifier nodes this can be done by scaling down the
 * frequencies.
 */

typedef enum node_type {
    LEAF = 0,
    BRANCH = 1
} ntype_t;

typedef struct bdt2_node bdt2_node_t;
typedef struct bayesian_delegation_tree2 bdt2_t;

struct bdt2_node {
    /** The struct members are rearranged pointers first in order to guarantee packing **/
    bdt2_t * root_bdt2;
    cnbp_t * classifier;
    cnbp_t * error_classifier;
    uint32_t children[BRANCH_NUM];
    ntype_t type;
    uint32_t node_id;
    uint32_t num_trained_total;
    uint32_t num_trained_last;
    double error_total;
    double error_last;
};

struct bayesian_delegation_tree2 {
    /** The struct members are rearranged pointers first in order to guarantee packing **/
    uint8_t * categories;
    bdt2_node_t ** nodes;
    double * class_scratch_space;
    double branch_scratch_space[BRANCH_NUM];
    size_t nodes_num;
    uint32_t cat_num;
    uint32_t split_threshold;
    uint32_t split_limit;
    uint32_t split_number;
    double nb_alpha;
    uint8_t use_probs;
    uint8_t class_num;
};

bdt2_t * create_bdt2(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        uint32_t split_threshold,
        uint32_t split_limit,
        double nb_alpha,
        uint8_t use_probs);

void bdt2_train_single(
        bdt2_t * bdt2,
        const uint8_t * data,
        const double * label);

void bdt2_train_batch(
        bdt2_t * bdt2,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize);

void bdt2_predict_class(
        const bdt2_t * bdt2,
        const uint8_t * data,
        double * labels);

void free_bdt2(
        bdt2_t * bdt2);

bdt2_t * bdt2_from_file(
        FILE * file,
        uint8_t use_probs);

bdt2_t * bdt2_from_file_with_name(
        const char * filename,
        uint8_t use_probs);

void bdt2_to_file(
        const bdt2_t * bdt2,
        FILE * file);

void bdt2_to_file_with_name(
        const bdt2_t * bdt2,
        const char * filename);

#endif
