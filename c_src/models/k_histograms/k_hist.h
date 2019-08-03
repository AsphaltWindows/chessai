#ifndef K_HIST_H_INCLUDED
#define K_HIST_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

typedef struct k_hists {
    uint8_t * categories;
    uint8_t cluster_num;
    uint8_t active_hists;
    uint8_t cat_num;
    uint32_t total_cat_val;
    uint32_t * cat_idx;
    uint32_t * cluster_hists;
    uint32_t * cluster_totals;
} khist_clust_t;

khist_clust_t * create_khist(const uint8_t * categories, size_t cat_num, uint8_t cluster_num, const uint8_t * const * start_hists, size_t smsize);

void free_khist(khist_clust_t *khist);

khist_clust_t * khist_model_from_vals(const uint32_t * values, size_t num_values);

uint32_t * khist_model_to_vals(const khist_clust_t * cnb);

void train_full(khist_clust_t *khist, const uint8_t * const * data, size_t dsize);

void train_incremental(khist_clust_t *khist, const uint8_t * const * data, size_t dsize);

double * assign_cluster(const khist_clust_t * khist, const uint8_t * data);

#endif
