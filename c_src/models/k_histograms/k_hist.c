#include "k_hist.h"

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

static void assign_cluster_with_mem(const khist_clust_t * khist, const uint8_t * data, uint8_t * res);

khist_clust_t * create_khist(const uint8_t * categories, size_t cat_num, uint8_t cluster_num, const uint8_t * const * start_hist, size_t smsize) {
    khist_clust_t * res;

    if (smsize > cluster_num) {
        printf("Failed to create K-Histogram clustering, because more starting histograms were provided than the expected number of clusters\n");
        return NULL;
    }

    if (!(res = malloc(sizeof(khist_clust_t)))) {
        printf("Failed to allocate memory for K-Histogram clustering\n");
        return NULL;
    }

    memset(res, 0, sizeof(khist_clust_t));

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

    for (unsigned int cat = 0; cat < cat_num; ++cat) {
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

    for (unsigned int m = 0; m < smsize; ++m) {
        memcpy(res->cluster_modes + (m * cat_num) * sizeof(uint8_t), start_modes[m], cat_num * sizeof(uint8_t));
    }

    res->cat_num = cat_num;
    res->cluster_num = cluster_num;
    res->active_hists = (uint8_t) smsize;

    return res;
}

void free_khist(khist_clust_t *khist) {

    if (khist->categories) {
        free(khist->categories);
        khist->categories = NULL;
    }

    if (khist->cat_idx) {
        free(khist->cat_idx);
        khist->cat_idx = NULL;
    }

    if (khist->cluster_modes) {
        free(khist->cluster_modes);
        khist->cluster_modes = NULL;
    }

    free(khist);
    return;
}

void train_batch(khist_clust_t *khist, const uint8_t * const * data, size_t dsize) {
    int iterations = 0;
    uint8_t cc[2];
    uint64_t last_cost, cost;
    uint8_t * tmp_modes;
    uint32_t mfreq;
    uint8_t mfreq_idx[256];
    uint8_t mfreq_count = 0;
    uint32_t * freqs;

    printf("K-Histogram clustering training on %zu data points\n", dsize);

    if (khist->cluster_num - khist->active_hists > dsize) {
        printf("Insufficient amount of data provided for K-modes clustering\n");
        return;
    }

    for (int cl = khist->active_hists; cl < khist->cluster_num; ++cl) {
        memcpy(khist->cluster_modes + cl * khist->cat_num * sizeof(uint8_t), data[(size_t) random() % dsize], khist->cat_num * sizeof(uint8_t));
        khist->active_hists++;
    }

    if (!(tmp_modes = malloc(khist->cluster_num * khist->cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-modes clustering temporary modes\n");
        return;
    }

    if (!(freqs = malloc(khist->cluster_num * khist->total_cat_val * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for K-modes clustering frequencies\n");
        return;
    }

    /** some debug **/
//    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//        printf("Mode %hhu: ", m);
//        for (uint8_t c = 0; c < khist->cat_num; ++c) {
//            printf("%hhu", khist->cluster_modes[m * khist->cat_num + c]);
//        }
//        printf("\n");
//    }
//    fflush(stdout);
//
    /** end debug **/
    while (1) {
        cost = 0;

        memset(freqs, 0, khist->cluster_num * khist->total_cat_val * sizeof(uint32_t));

        for (unsigned int didx = 0; didx < dsize; ++didx) {
            assign_cluster_with_mem((const khist_clust_t *)khist, data[didx], (uint8_t *) cc);
            cost += cc[1];
//            printf("c_cost:%u, cluster_num:%hhu, cluster_mode:", cc[1], cc[0]);
//            for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
//                printf("%hhu", khist->cluster_modes[cc[0] * khist->cat_num + cat]);
//            }
//            printf(", data point:");
//            for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
//                printf("%hhu", data[didx][cat]);
//            }
//            printf("\n");

//            printf("Vector: ");
//            for (uint8_t c = 0; c < khist->cat_num; ++c) {
//                printf("%hhu ", data[didx][c]);
//            }
//            printf("\n");
//
//            printf("Assigned to mode: ");
//            for (uint8_t c = 0; c < khist->cat_num; ++c) {
//                printf("%hhu ", khist->cluster_modes[cc[0]* khist->cat_num + c]);
//            }
//            printf("\n");
//            printf("Cost: %hhu\n", cc[1]);

//            printf("c_cat_num: %hhu  total_cat_val: %u \n", khist->cat_num, khist->total_cat_val);
            for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
//                printf("c_catidx %d: %u\n", cat, khist->cat_idx[cat]);
//                printf("c_incrementing freq at index: %u\n", cc[0] * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]);
                freqs[cc[0] * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] += 1;
            }
        }

        /** some debug **/
//        for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//            printf("Mode %hhu: ", m);
//            for (uint8_t c = 0; c < khist->cat_num; ++c) {
//                printf("%hhu", khist->cluster_modes[m * khist->cat_num + c]);
//            }
//            printf("\n");
//        }
//        fflush(stdout);

//        printf("c_freqs:\n");
//        for (uint8_t cl = 0; cl < khist->cluster_num; ++cl) {
//            printf("c_freq %hhu: ", cl);
//            for (uint8_t cat = 0; cat < khist->cat_num; cat++) {
//                for (uint8_t val = 0; val < khist->categories[cat]; ++val) {
//                    printf("%u", freqs[cl * khist->total_cat_val + khist->cat_idx[cat] + val]);
//                }
//            }
//            printf("\n");
//        }

        /** end debug **/
        if (iterations >= 1 && cost >= last_cost) {
            printf("Last cost: %lu, cost: %lu\n", last_cost, cost);
            break;
        }

        ++iterations;
        last_cost = cost;

        printf("Finished clustering attempt number: %d, cost: %lu\n", iterations, cost);

        /** some debug **/
//        for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//            printf("c_mode %hhu: ", m);
//            for (uint8_t c = 0; c < khist->cat_num; ++c) {
//                printf("%hhu", khist->cluster_modes[m * khist->cat_num + c]);
//            }
//            printf("\n");
//        }
//        fflush(stdout);

        /** end debug **/

        memcpy(tmp_modes, khist->cluster_modes, khist->cluster_num * khist->cat_num * sizeof(uint8_t));

        for (uint8_t cl = 0; cl < khist->cluster_num; ++cl) {
            for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
                memset(mfreq_idx, 0, sizeof(mfreq_idx));
                mfreq_count = 0;
                mfreq = 0;
                for (uint8_t val = 0; val < khist->categories[cat]; ++val) {
                    uint32_t f = freqs[cl * khist->total_cat_val + khist->cat_idx[cat] + val];
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

                uint8_t mrandval = (uint8_t) random() % mfreq_count;
                khist->cluster_modes[cl * khist->cat_num + cat] = mfreq_count > 1 ?
                    //mfreq_idx[0] :
                    mfreq_idx[0] :
                    mfreq_idx[0];
//                printf("Max frequency for cluster: %hhu, category: %hhu is %u, with value: %hhu\n", cl, cat, mfreq, mfreq_idx[mrandval]);
            }
        }

        /** some debug **/
//        for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//            printf("c_mode %hhu: ", m);
//            for (uint8_t c = 0; c < khist->cat_num; ++c) {
//                printf("%hhu", khist->cluster_modes[m * khist->cat_num + c]);
//            }
//            printf("\n");
//        }
//        fflush(stdout);

        /** end debug **/

    }

    memcpy(khist->cluster_modes, tmp_modes, khist->cluster_num * khist->cat_num * sizeof(uint8_t));

    /** some debug **/
    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
        printf("Mode %hhu: ", m);
        for (uint8_t c = 0; c < khist->cat_num; ++c) {
            printf("%hhu", khist->cluster_modes[m * khist->cat_num + c]);
        }
        printf("\n");
    }
    fflush(stdout);

    /** end debug **/

    free(tmp_modes);
    free(freqs);

    return;
}

static void assign_cluster_with_mem(const khist_clust_t * khist, const uint8_t * data, uint8_t * res) {
    uint8_t cost;
    uint8_t mincost;
    uint8_t mincost_idx[256];
    uint8_t mincost_count;


    mincost_count = 0;
    memset(mincost_idx, 0, sizeof(mincost_idx));
    for (int mod = 0; mod < khist->cluster_num; ++mod) {
        cost = 0;

        for (int cat = 0; cat < khist->cat_num; ++cat) {

            if (data[cat] != khist->cluster_modes[mod * khist->cat_num + cat]) {
                cost++;
            }

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

    uint8_t mrandval = (uint8_t) random() % mincost_count;
    res[0] = mincost_count > 1 ?
          mincost_idx[0] :
//        mincost_idx[(uint8_t) random() % mincost_count] :
        mincost_idx[0];
    res[1] = mincost;

    return;
}

uint8_t * assign_cluster(const khist_clust_t * khist, const uint8_t * data) {
    uint8_t * res;

    if (!(res = malloc(2 * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-modes clustering cluster and index\n");
        return NULL;
    }

    assign_cluster_with_mem(khist, data, res);
    return res;
}

khist_clust_t * khist_model_from_vals(const uint32_t * values, size_t num_values) {
    uint8_t cluster_num;
    uint8_t cat_num;
    uint8_t * categories;
    khist_clust_t * res;
    size_t at = 0;

    if (num_values < 2) {
        printf("Insufficient values to load K-modes clustering model\n");
        return NULL;
    }

    cat_num = (uint8_t) values[at++];
    cluster_num = (uint8_t) values[at++];

    printf("Loading K-Histogram Clustering with %hhu categories and %hhu clusters\n", cat_num, cluster_num);

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

    if (!(res = create_khist((const uint8_t *) categories, cat_num, cluster_num, NULL, 0))) {
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

    res->active_hists = cluster_num;
    res->cat_num = cat_num;
    res->cluster_num = cluster_num;

    return res;
}

uint32_t * khist_model_to_vals(const khist_clust_t * khist) {
    size_t at = 0;
    size_t val_num;
    uint32_t * res;

    val_num = 2 + khist->cat_num + khist->cat_num * khist->cluster_num;

    if (!(res = malloc(val_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory to serialize K-modes clustering model\n");
        return NULL;
    }

    res[at++] = khist->cat_num;
    res[at++] = khist->cluster_num;

    for (int cat = 0; cat < khist->cat_num; ++cat) {
        res[at++] = khist->categories[cat];
    }

    for (int cl = 0; cl < khist->cluster_num; ++cl) {
        for (int cat = 0; cat < khist->cat_num; ++cat) {
            res[at++] = khist->cluster_modes[cl * khist->cat_num + cat];
        }
    }

    return res;
}
