#ifndef K_HIST_H_INCLUDED
#define K_HIST_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

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

khist_clust_t * create_khist(
        const uint8_t * categories,
        size_t cat_num,
        uint8_t cluster_num,
        const uint8_t * const * start_hists,
        size_t smsize);

void free_khist(khist_clust_t *khist);

khist_clust_t * khist_model_from_vals(
        const uint32_t * values,
        size_t num_values);

uint32_t * khist_model_to_vals(
        const khist_clust_t * cnb);

void train_full(
        khist_clust_t *khist,
        const uint8_t * const * data,
        size_t dsize);

void train_full_on_selected(
        khist_clust_t *khist,
        const uint8_t * const * data,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected);

void train_incremental(
        khist_clust_t *khist,
        const uint8_t * const * data,
        size_t dsize);

void train_incremental_on_selected(
        khist_clust_t *khist,
        const uint8_t * const * data,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected);

double * assign_cluster(
        const khist_clust_t * khist,
        const uint8_t * data);

uint8_t assign_cluster_no_cost(
        const khist_clust_t * khist,
        const uint8_t * data);

khist_clust_t * khist_from_file(
        FILE * file);

void khist_to_file(
        khist_clust_t *khist,
        FILE * file);

#endif
