#include <stddef.h>
#include <stdint.h>

typedef struct k_modes {
    uint8_t * categories;
    uint8_t cat_num;
    uint32_t total_cat_val;
    uint32_t * cat_idx;
    uint8_t * cluster_modes;
    uint8_t cluster_num;
    uint8_t active_modes;
} kmodes_clust_t;

kmodes_clust_t * create_kmodes(const uint8_t * categories, size_t cat_num, uint8_t cluster_num, const uint8_t * const * start_modes, size_t smsize);

void free_kmodes(kmodes_clust_t *kmodes);

kmodes_clust_t * kmodes_model_from_vals(const uint32_t * values, size_t num_values);

uint32_t * kmodes_model_to_vals(const kmodes_clust_t * cnb);

void train_batch(kmodes_clust_t *kmodes, const uint8_t * const * data, size_t dsize);

uint8_t * assign_cluster(const kmodes_clust_t * kmodes, const uint8_t * data);

