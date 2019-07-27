#include "k_modes.h"

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

static void assign_cluster_with_mem(const kmodes_clust_t * kmodes, const uint8_t * data, uint8_t ** res);

kmodes_clust_t * create_kmodes(const uint8_t * categories, size_t cat_num, uint8_t cluster_num, const uint8_t * const * start_modes, size_t smsize) {
    kmodes_clust_t * res;    

    if (smsize > cluster_num) {
        printf("Failed to create K-modes clustering, because more starting modes were provided than the expected number of clusters\n");
        return NULL;
    }

    if (!(res = malloc(sizeof(kmodes_clust_t)))) {
        printf("Failed to allocate memory for K-modes clustering\n");
        return NULL;
    }

    memset(res, 0, sizeof(kmodes_clust_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-modes clustering categories\n");
        return NULL;
    }

    memset(res->categories, 0, cat_num * sizeof(uint8_t));

    if (!(res->cat_idx = malloc(cat_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for K-modes clustering category indices\n");
        return NULL;
    }

    memset(res->cat_idx, 0, cat_num * sizeof(uint32_t));
    res->total_cat_val = 0;

    for (int cat = 0; cat < cat_num; ++cat) {
        res->total_cat_val += categories[cat];
        res->categories[cat] = categories[cat];

        if (cat > 0) {
            res->cat_idx[cat] = res->cat_idx[cat - 1] + categories[cat - 1];
        }
    }

    if (!(res->cluster_modes = malloc(cluster_num * cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-modes clustering modes\n");
        return NULL;
    }

    for (int m = 0; m < smsize; ++m) {
        memcpy(res->cluster_modes + (m * cat_num) * sizeof(uint8_t), start_modes[m], cat_num * sizeof(uint8_t));
    }

    res->cluster_num = cluster_num;
    res->active_modes = smsize;

    return res;
}

void free_kmodes(kmodes_clust_t *kmodes) {

    if (kmodes->categories) {
        free(kmodes->categories);
        kmodes->categories = NULL;
    }    

    if (kmodes->cat_idx) {
        free(kmodes->cat_idx);
        kmodes->cat_idx = NULL;
    }    

    if (kmodes->cluster_modes) {
        free(kmodes->cluster_modes);
        kmodes->cluster_modes = NULL;
    }    

    free(kmodes);
    return;
}

void train_batch(kmodes_clust_t *kmodes, const uint8_t * const * data, size_t dsize) {
    int iterations = 0;
    uint8_t cc[2];
    uint64_t last_cost, cost;
    uint8_t * tmp_modes;
    uint32_t mfreq;
    uint8_t mfreq_idx[256];
    uint8_t mfreq_count = 0;
    uint32_t * freqs;

    if (kmodes->cluster_num - kmodes->active_modes > dsize) {
        printf("Insufficient amount of data provided for K-modes clustering\n");
        return;
    }

    for (int cl = kmodes->active_modes; cl < kmodes->cluster_num; ++cl) {
        memcpy(kmodes->cluster_modes + cl * kmodes->cat_num * sizeof(uint8_t), data[(size_t) random() % dsize], kmodes->cat_num * sizeof(uint8_t));
        kmodes->active_modes++;
    }

    if (!(tmp_modes = malloc(kmodes->cluster_num * kmodes->cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-modes clustering temporary modes\n");
        return;
    }

    if (!(freqs = malloc(kmodes->cluster_num * kmodes->total_cat_val * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for K-modes clustering frequencies\n");
        return;
    }

    while (1) {
        cost = 0;

        memset(freqs, 0, kmodes->cluster_num * kmodes->total_cat_val * sizeof(uint32_t));

        for (int didx = 0; didx < dsize; ++didx) {
            assign_cluster_with_mem((const kmodes_clust_t *)kmodes, data[didx], (uint8_t **) &cc);
            cost += cc[1];

            for (int cat = 0; cat < kmodes->cat_num; ++cat) {
                freqs[cc[0] * kmodes->total_cat_val + kmodes->cat_idx[cat] + data[didx][cat]] += 1;
            }
        }

        if (iterations >= 1 && cost >= last_cost) {
            break;
        }

        ++iterations;
        last_cost = cost;

        memcpy(tmp_modes, kmodes->cluster_modes, kmodes->cluster_num * kmodes->cat_num * sizeof(uint8_t));

        for (int cl = 0; cl < kmodes->cluster_num; ++cl) {
            for (int cat = 0; cat < kmodes->cat_num; ++cat) {
                memset(mfreq_idx, 0, sizeof(mfreq_idx));
                mfreq_count = 0;
                mfreq = 0;
                for (int val = 0; val < kmodes->categories[cat]; ++val) {
                    int f = freqs[cl * kmodes->total_cat_val + kmodes->cat_idx[cat] + val];
                    if (f > mfreq || !mfreq_count) {
                        memset(mfreq_idx, 0, sizeof(mfreq_idx));
                        mfreq_idx[0] = val;
                        mfreq_count = 1;
                        mfreq = f;
                    }
                    else if (f == mfreq) {
                        mfreq_idx[mfreq_count++] = val;
                    }
                }

                kmodes->cluster_modes[cl * kmodes->total_cat_val + cat] = mfreq_count > 1 ?
                    mfreq_idx[(uint8_t) random() % mfreq_count] :
                    mfreq_idx[0];
            }
        }
    }

    memcpy(kmodes->cluster_modes, tmp_modes, kmodes->cluster_num * kmodes->cat_num * sizeof(uint8_t));

    free(tmp_modes);
    free(freqs);

    return;
}

static void assign_cluster_with_mem(const kmodes_clust_t * kmodes, const uint8_t * data, uint8_t ** res) {
    uint8_t cost;
    uint8_t mincost;
    uint8_t mincost_idx[256];
    uint8_t mincost_count;

    mincost_count = 0;
    memset(mincost_idx, 0, sizeof(mincost_idx));
    for (int mod = 0; mod < kmodes->cluster_num; ++mod) {
        cost = 0;

        for (int cat = 0; mod < kmodes->cat_num; ++cat) {
            cost += data[cat] ^ kmodes->cluster_modes[mod * kmodes->cat_num + cat] ? 1 : 0;
        }

        if (cost < mincost || !mincost_count) {
            mincost = cost;
            memset(mincost_idx, 0, sizeof(mincost_idx));
            mincost_idx[0] = mod;
            mincost_count = 1;
        }
        else if (cost == mincost) {
            mincost_idx[mincost_count++] = mod;
        }
    }

    (*res)[0] = mincost_count > 1 ?
        mincost_idx[(uint8_t) random() % mincost_count] :
        mincost_idx[0];
    (*res)[0] = mincost;

    return;
}

uint8_t * assign_cluster(const kmodes_clust_t * kmodes, const uint8_t * data) {
    uint8_t * res;

    if (!(res = malloc(2 * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-modes clustering cluster and index\n");
        return NULL;
    }

    assign_cluster_with_mem(kmodes, data, &res);
    return res;
}

kmodes_clust_t * kmodes_model_from_vals(const uint32_t * values, size_t num_values) {
    uint8_t cluster_num;
    uint8_t cat_num;
    uint8_t * categories;
    kmodes_clust_t * res;
    size_t at = 0;

    if (num_values < 2) {
        printf("Insufficient values to load K-modes clustering model\n");
        return NULL;
    }

    cat_num = (uint8_t) values[at++];
    cluster_num = (uint8_t) values[at++];

    if (num_values < at + cat_num) {
        printf("Insufficient values to load K-modes clustering model\n");
        return NULL;
    }

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-modes clustering model categories\n");
        return NULL;
    }

    for (int cat = 0; cat < cat_num; ++cat) {
        categories[cat] = (uint8_t) values[at++];
    }

    if (!(res = create_kmodes((const uint8_t *) categories, cat_num, cluster_num, NULL, 0))) {
        printf("Failed to create K-modes clustering model\n");
        free(categories);
        return NULL;
    }

    free(categories);

    if (num_values < at + cat_num * cluster_num) {
        printf("Insufficient values to load K-modes clustering model\n");
        return NULL;
    }

    for (int cl = 0; cl < cluster_num; ++cl) {
        for(int cat = 0; cat < cat_num; ++cat) {
            res->cluster_modes[cl * cat_num + cat] = (uint8_t) values[at++];
        }
    }

    res->active_modes = cluster_num;

    return res;
}

uint32_t * kmodes_model_to_vals(const kmodes_clust_t * kmodes) {
    size_t at = 0;
    size_t val_num;
    uint32_t * res;

    val_num = 2 + kmodes->cat_num + kmodes->cat_num * kmodes->cluster_num;

    if (!(res = malloc(val_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory to serialize K-modes clustering model\n");
        return NULL;
    }

    res[at++] = kmodes->cat_num;
    res[at++] = kmodes->cluster_num;

    for (int cat = 0; cat < kmodes->cat_num; ++cat) {
        res[at++] = kmodes->categories[cat];
    }

    for (int cl = 0; cl < kmodes->cluster_num; ++cl) {
        for (int cat = 0; cat < kmodes->cat_num; ++cat) {
            res[at++] = kmodes->cluster_modes[cl * kmodes->cat_num + cat];
        }
    }

    return res;
}
