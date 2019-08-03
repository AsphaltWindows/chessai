#include "k_hist.h"

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

static void assign_cluster_with_mem(const khist_clust_t * khist, const uint8_t * data, uint8_t * res);

khist_clust_t * create_khist(const uint8_t * categories, size_t cat_num, uint8_t cluster_num, const uint8_t * const * start_hist, size_t smsize) {
    khist_clust_t * res;

    if (smsize > cluster_num) {
        printf("Failed to create K-Histograms clustering, because more starting histograms were provided than the expected number of clusters\n");
        return NULL;
    }

    if (!(res = malloc(sizeof(khist_clust_t)))) {
        printf("Failed to allocate memory for K-Histograms clustering\n");
        return NULL;
    }

    memset(res, 0, sizeof(khist_clust_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-Histograms clustering categories\n");
        return NULL;
    }

    memset(res->categories, 0, cat_num * sizeof(uint8_t));

    if (!(res->cat_idx = malloc(cat_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for K-Histograms clustering category indices\n");
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

    if (!(res->cluster_hists = malloc(cluster_num * res->total_cat_val * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for K-Histograms clustering histograms\n");
        return NULL;
    }

    memset(res->cluster_hists, 0, cluster_num * res->total_cat_val * sizeof(uint32_t));

    if (!(res->cluster_totals = malloc(cluster_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for K-Histograms cluster totals\n");
        return NULL;
    }

    memset(res->cluster_totals, 0, cluster_num * sizeof(uint32_t));

    for (unsigned int h = 0; h < smsize; ++h) {
        for (uint8_t cat = 0; cat < res->cat_num; ++cat) {
            res->cluster_hists[h * res->total_cat_val + res->cat_idx[cat] + start_hist[h][cat]] = 1;
        }
        res->cluster_totals[h] = 1;
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

    if (khist->cluster_hists) {
        free(khist->cluster_modes);
        khist->cluster_modes = NULL;
    }

    if (khist->cluster_totals) {
        free(khist->cluster_totals);
        khist->cluster_totals = NULL;
    }

    free(khist);
    return;
}

void train_full(khist_clust_t *khist, const uint8_t * const * data, size_t dsize) {
    int iterations = 0;
    double cc[2];
    double cost;
    uint32_t * dlabels;

    printf("K-Histograms clustering training on %zu data points\n", dsize);

    if (khist->cluster_num - khist->active_hists > dsize) {
        printf("Insufficient amount of data provided for K-Histograms clustering\n");
        return;
    }

    for (int cl = khist->active_hists; cl < khist->cluster_num; ++cl) {
        memcpy(khist->cluster_modes + cl * khist->cat_num * sizeof(uint8_t), data[(size_t) random() % dsize], khist->cat_num * sizeof(uint8_t));
        khist->active_hists++;
    }

    /** some debug **/
//    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//        printf("Mode %hhu: ", m);
//        for (uint8_t c = 0; c < khist->cat_num; ++c) {
// //            printf("%hhu", khist->cluster_hists[m * khist->cat_num + c]);
//        }
//        printf("\n");
//    }
//    fflush(stdout);
//
    /** end debug **/

    /** original assignment **/

    if (!(dlabels = malloc(dsize * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-Histograms training labels\n");
        return;
    }

    memset(dlabels, 0, dsize * sizeof(uint8_t));

    printf("Performing initial K-Histogram clustering\n");
    /** initial labeling **/
    cost = 0.0;
    for (size_t didx = 0; d < dsize; ++d) {
        assign_cluster_with_mem((const khist_clust_t *)khist, data[didx], (double *) cc);

        dlabels[didx] = (uint8_t) cc[0];
        cost += cc[1];

        for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
            khist->cluster_hists[((uint8_t)cc[0]) * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] += 1;
        }
    }

    printf("Initial K-Histogram clustering achieved with cost: %f\n", cost);

    /** Relabeling until maximum fit **/
    size_t reassigned_num = 0;
    iterations = 0;
    while (!reassigned_num) {
        reassigned_num = 0;
        cost = 0;

        for (size_t didx = 0; d < dsize; ++d) {
            assign_cluster_with_mem((const khist_clust_t *) khist, data[didx], (double *) cc);

            uint8_t old_label = dlabels[didx];
            uint8_t new_label = (uint8_t) cc[0];
            cost += cc[1];

            if (new_label != old_label) {
                ++reassigned_num;
                dlabels[didx] = new_label;

                for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
                    khist->cluster_hists[old_label * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] -= 1;
                    khist->cluster_hists[new_label * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] += 1;
                }
            }
        }

        ++iterations;

        printf("Completed K-Histogram reclustering, %zu data points changed labels, total cost is %f\n",
               reassigned_num,
               cost);

    }

    free(dlabels);
    return;
}

void train_incremental(khist_clust_t *khist, const uint8_t * const * data, size_t dsize) {

    double cc[2];
    double cost;

    printf("K-Histograms incremental training on %zu data points\n", dsize);

    cost = 0.0;
    for (size_t didx = 0; d < dsize; ++d) {
        assign_cluster_with_mem((const khist_clust_t *)khist, data[didx], (double *) cc);

        cost += cc[1];

        for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
            khist->cluster_hists[((uint8_t)cc[0]) * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] += 1;
        }
    }

    printf("Completed K-Histogram clustering achieved for %zu data points with cost: %f\n", dsize, cost);

    return;
}

static void assign_cluster_with_mem(const khist_clust_t * khist, const uint8_t * data, double * res) {
    double cost;
    double mincost;
    uint8_t mincost_idx[256];
    uint8_t mincost_count;


    mincost_count = 0;
    memset(mincost_idx, 0, sizeof(mincost_idx));
    for (int hist = 0; hist < khist->cluster_num; ++hist) {
        cost = 0;

        for (int cat = 0; cat < khist->cat_num; ++cat) {
            if (data[cat] != khist->cluster_modes[mod * khist->cat_num + cat]) {
                cost += ((double) (khist->cluster_totals[hist]) - (double)(khist->cluster_hists[hist*khist->total_cat_val + khist->cat_idx[cat] + data[cat]]);
            }
        }

        cost /= (double) (khist->cluster_totals[hist]);

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
    res[0] = (double) (mincost_count > 1 ?
          mincost_idx[0] :
//        mincost_idx[(uint8_t) random() % mincost_count] :
        mincost_idx[0]);
    res[1] = mincost;

    return;
}

double * assign_cluster(const khist_clust_t * khist, const uint8_t * data) {
    uint8_t * res;

    if (!(res = malloc(2 * sizeof(double)))) {
        printf("Failed to allocate memory for K-Histograms clustering cluster and index\n");
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
        printf("Insufficient values to load K-Histograms clustering model\n");
        return NULL;
    }

    cat_num = (uint8_t) values[at++];
    cluster_num = (uint8_t) values[at++];

    printf("Loading K-Histograms Clustering with %hhu categories and %hhu clusters\n", cat_num, cluster_num);

    if (num_values < at + cat_num) {
        printf("Insufficient values to load K-Histograms clustering model\n");
        return NULL;
    }

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-Histograms clustering model categories\n");
        return NULL;
    }

    for (int cat = 0; cat < cat_num; ++cat) {
        categories[cat] = (uint8_t) values[at++];
    }

    if (!(res = create_khist((const uint8_t *) categories, cat_num, cluster_num, NULL, 0))) {
        printf("Failed to create K-Histograms clustering model\n");
        free(categories);
        return NULL;
    }

    free(categories);

    if (num_values < at + cluster_num) {
        printf("Insufficient values to load K-Histogram cluster totals\n");
        return NULL;
    }

    for (int cl = 0; cl < cluster_num; ++cl) {
        res->cluster_totals[cl] == values[at++];
    }

    if (num_values < at + cluster_num * res->total_cat_val) {
        printf("Insufficient values to load K-Histograms clustering model\n");
        return NULL;
    }

    for (int cl = 0; cl < cluster_num; ++cl) {
        for(int cat = 0; cat < cat_num; ++cat) {
            for (int val = 0; val < res->categories[cat]; ++val) {
                res->cluster_hists[cl * res->total_cat_val + res->cat_idx[cat] + val] = values[at++];
            }
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

    val_num = 2 + khist->cat_num + khist->cluster_num + khist->cluster_num * khist->total_cat_val;

    if (!(res = malloc(val_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory to serialize K-Histograms clustering model\n");
        return NULL;
    }

    res[at++] = khist->cat_num;
    res[at++] = khist->cluster_num;

    for (int cat = 0; cat < khist->cat_num; ++cat) {
        res[at++] = khist->categories[cat];
    }

    for (int cl = 0; cl < khist->cluster_num; ++cl) {
        res[at++] = khist->cluster_totals[cl];
    }

    for (int cl = 0; cl < khist->cluster_num; ++cl) {
        for (int cat = 0; cat < khist->cat_num; ++cat) {
            for (int val = 0; val < khist->categories[cat]; ++val) {
                res[at++] = khist->cluster_hists[cl * khist->total_cat_val + khist->cat_idx[cat] + val];
            }
        }
    }

    return res;
}
