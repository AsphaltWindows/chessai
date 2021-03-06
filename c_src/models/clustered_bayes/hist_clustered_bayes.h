#ifndef HIST_CLUSTERED_BAYES_H_INCLUDED
#define HIST_CLUSTERED_BAYES_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

#include "../naive_bayes/categorical_naive_bayes.h"
#include "../k_histograms/k_hist.h"

#define LEAF 0
#define BRANCH 1

typedef struct hist_clustered_bayes_node {
    uint8_t class_num;
    uint8_t cat_num;
    uint8_t * categories;
    uint32_t hierarchy_size;
    uint8_t *hierarchy;
    uint8_t node_type;
    uint8_t alpha;
    uint8_t forget_factor;
    uint32_t limit;
    uint32_t num_trained;
    cnb_clas_t *classifier;
    uint8_t cluster_num;
    khist_clust_t *clustering;
    struct hist_clustered_bayes_node ** children;
} hcb_node_t;

typedef struct hist_clustered_bayes {
    uint8_t class_num;
    uint8_t cat_num;
    uint8_t * categories;
    uint8_t alpha;
    uint8_t forget_factor;
    uint32_t limit;
    uint8_t cluster_num;
    hcb_node_t * root;
} hcb_clas_t;

void train_model(
        hcb_clas_t *hcb,
        const uint8_t * const * data,
        const uint8_t * labels,
        size_t dsize);

double * hcb_predict_class(
        const hcb_clas_t * hcb,
        const uint8_t * data);

hcb_clas_t * create_hcb(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        uint8_t cluster_num,
        double alpha,
        uint32_t limit,
        uint8_t forget_factor);

hcb_clas_t * hcb_model_from_file(
        const char * filename);

void hcb_model_to_file(
        const hcb_clas_t * hcb,
        const char * filename);

void free_hcb(
        hcb_clas_t *hcb);

#endif
