#include "k_hist.h"

#include <stdlib.h>
#include <string.h>

static void assign_cluster_with_mem(const khist_clust_t * khist, const uint8_t * data, double * res);

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

    res->cat_num = cat_num;
    res->cluster_num = cluster_num;

    for (size_t h = 0; h < smsize; ++h) {
        for (uint8_t cat = 0; cat < res->cat_num; ++cat) {
            res->cluster_hists[h * res->total_cat_val + res->cat_idx[cat] + start_hist[h][cat]] = 1;
        }
        res->cluster_totals[h] = 1;
    }

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
        free(khist->cluster_hists);
        khist->cluster_hists = NULL;
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
    uint8_t * dlabels;

    printf("K-Histograms clustering training on %zu data points\n", dsize);
    fflush(stdout);

    if (khist->cluster_num - khist->active_hists > dsize) {
        printf("Insufficient amount of data provided for K-Histograms clustering\n");
        return;
    }

    for (int cl = khist->active_hists; cl < khist->cluster_num; ++cl) {
        size_t rand_hist_centroid_idx = (size_t) (random() % dsize);

        for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
            khist->cluster_hists[cl * khist->total_cat_val + khist->cat_idx[cat] + data[rand_hist_centroid_idx][cat]] += 1;
        }

        khist->cluster_totals[cl] = 1;
        khist->active_hists++;
    }

//    /** some debug **/
//    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//
//        printf("Mode %hhu:\n", m);
//        for (uint8_t c = 0; c < khist->cat_num; ++c) {
//            printf("Category %hhu: ", c);
//            for (uint8_t val = 0; val < khist->categories[c]; ++val) {
//                printf("%u", khist->cluster_hists[m * khist->total_cat_val + khist->cat_idx[c] + val]);
//            }
//            printf("\n");
//        }
//        printf("\n");
//    }
//    fflush(stdout);

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
    for (size_t didx = 0; didx < dsize; ++didx) {
        assign_cluster_with_mem((const khist_clust_t *)khist, data[didx], (double *) cc);

//        printf("%zu, assigned label: %hhu, cost: %f\n", didx, (uint8_t) cc[0], cc[1]);

        dlabels[didx] = (uint8_t) cc[0];
        khist->cluster_totals[(uint8_t) cc[0]] += 1;
        cost += cc[1];

        for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
            khist->cluster_hists[((uint8_t)cc[0]) * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] += 1;
        }
    }

    /** some debug **/
//    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//
//        printf("Mode %hhu:\n", m);
//        for (uint8_t c = 0; c < khist->cat_num; ++c) {
//            printf("Category %hhu: ", c);
//            for (uint8_t val = 0; val < khist->categories[c]; ++val) {
//                printf("%u", khist->cluster_hists[m * khist->total_cat_val + khist->cat_idx[c] + val]);
//            }
//            printf("\n");
//        }
//        printf("\n");
//    }
//    fflush(stdout);

    /** end debug **/
    printf("Initial K-Histogram clustering achieved with cost: %f\n", cost);

    /** Relabeling until maximum fit **/
    size_t reassigned_num = 1;
    iterations = 0;
    while (reassigned_num) {
        reassigned_num = 0;
        cost = 0;

        for (size_t didx = 0; didx < dsize; ++didx) {
            assign_cluster_with_mem((const khist_clust_t *) khist, data[didx], (double *) cc);

            uint8_t old_label = dlabels[didx];
            uint8_t new_label = (uint8_t) cc[0];
            cost += cc[1];

//            printf("%zu, old label: %hhu, new label: %hhu, cost: %f\n", didx, old_label, new_label, cc[1]);

            if (new_label != old_label) {
//                printf("Reassigning data point from label: %hhu, to label: %hhu\n", old_label, new_label);
                ++reassigned_num;
                dlabels[didx] = new_label;
                khist->cluster_totals[old_label] -= 1;
                khist->cluster_totals[new_label] += 1;

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

//        /** some debug **/
//        for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//
//            printf("Mode %hhu:\n", m);
//            for (uint8_t c = 0; c < khist->cat_num; ++c) {
//                printf("Category %hhu: ", c);
//                for (uint8_t val = 0; val < khist->categories[c]; ++val) {
//                    printf("%u", khist->cluster_hists[m * khist->total_cat_val + khist->cat_idx[c] + val]);
//                }
//                printf("\n");
//            }
//            printf("\n");
//        }
//        fflush(stdout);

        /** end debug **/

    }

//    /** some debug **/
//    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//
//        printf("Mode %hhu:\n", m);
//        for (uint8_t c = 0; c < khist->cat_num; ++c) {
//            printf("Category %hhu: ", c);
//            for (uint8_t val = 0; val < khist->categories[c]; ++val) {
//                printf("%u", khist->cluster_hists[m * khist->total_cat_val + khist->cat_idx[c] + val]);
//            }
//            printf("\n");
//        }
//        printf("\n");
//    }
//    fflush(stdout);
//
//    /** end debug **/

    free(dlabels);
    return;
}


void train_full_on_selected(
        khist_clust_t *khist,
        const uint8_t * const * data,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected)
{
    int iterations = 0;
    double cc[2];
    double cost;
    size_t didx;
    uint8_t * dlabels;

    printf("K-Histograms clustering training on %zu data points\n", num_selected);
    fflush(stdout);

    if (dsize < num_selected) {
        printf("The data size (%zu) is lesser than the number of selected data points (%zu), so there must be an error\n", dsize, num_selected);
        return;
    }

    if (khist->cluster_num - khist->active_hists > num_selected) {
        printf("Insufficient amount of data provided for K-Histograms clustering, but will proceed anyways\n");
    }

    for (int cl = khist->active_hists; cl < khist->cluster_num; ++cl) {
        size_t rand_hist_centroid_idx = selected_indices[(size_t) (random() % num_selected)];

        for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
            khist->cluster_hists[cl * khist->total_cat_val + khist->cat_idx[cat] + data[rand_hist_centroid_idx][cat]] += 1;
        }

        khist->cluster_totals[cl] = 1;
        khist->active_hists++;
    }

//    /** some debug **/
//    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//
//        printf("Mode %hhu:\n", m);
//        for (uint8_t c = 0; c < khist->cat_num; ++c) {
//            printf("Category %hhu: ", c);
//            for (uint8_t val = 0; val < khist->categories[c]; ++val) {
//                printf("%u", khist->cluster_hists[m * khist->total_cat_val + khist->cat_idx[c] + val]);
//            }
//            printf("\n");
//        }
//        printf("\n");
//    }
//    fflush(stdout);

    /** end debug **/

    /** original assignment **/

    if (!(dlabels = malloc(num_selected * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-Histograms training labels\n");
        return;
    }

    memset(dlabels, 0, num_selected * sizeof(uint8_t));

    printf("Performing initial K-Histogram clustering\n");
    /** initial labeling **/
    cost = 0.0;
    for (size_t sel_idx = 0; sel_idx < num_selected; ++sel_idx) {
        didx = selected_indices[sel_idx];
        assign_cluster_with_mem((const khist_clust_t *)khist, data[didx], (double *) cc);

//        printf("%zu, assigned label: %hhu, cost: %f\n", didx, (uint8_t) cc[0], cc[1]);

        dlabels[didx] = (uint8_t) cc[0];
        khist->cluster_totals[(uint8_t) cc[0]] += 1;
        cost += cc[1];

        for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
            khist->cluster_hists[((uint8_t)cc[0]) * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] += 1;
        }
    }

    /** some debug **/
//    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//
//        printf("Mode %hhu:\n", m);
//        for (uint8_t c = 0; c < khist->cat_num; ++c) {
//            printf("Category %hhu: ", c);
//            for (uint8_t val = 0; val < khist->categories[c]; ++val) {
//                printf("%u", khist->cluster_hists[m * khist->total_cat_val + khist->cat_idx[c] + val]);
//            }
//            printf("\n");
//        }
//        printf("\n");
//    }
//    fflush(stdout);

    /** end debug **/
    printf("Initial K-Histogram clustering achieved with cost: %f\n", cost);

    /** Relabeling until maximum fit **/
    size_t reassigned_num = 1;
    iterations = 0;
    while (reassigned_num) {
        reassigned_num = 0;
        cost = 0;

        for (size_t sel_idx = 0; sel_idx < num_selected; ++sel_idx) {
            didx = selected_indices[sel_idx];
            assign_cluster_with_mem((const khist_clust_t *) khist, data[didx], (double *) cc);

            uint8_t old_label = dlabels[didx];
            uint8_t new_label = (uint8_t) cc[0];
            cost += cc[1];

//            printf("%zu, old label: %hhu, new label: %hhu, cost: %f\n", didx, old_label, new_label, cc[1]);

            if (new_label != old_label) {
//                printf("Reassigning data point from label: %hhu, to label: %hhu\n", old_label, new_label);
                ++reassigned_num;
                dlabels[didx] = new_label;
                khist->cluster_totals[old_label] -= 1;
                khist->cluster_totals[new_label] += 1;

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

//        /** some debug **/
//        for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//
//            printf("Mode %hhu:\n", m);
//            for (uint8_t c = 0; c < khist->cat_num; ++c) {
//                printf("Category %hhu: ", c);
//                for (uint8_t val = 0; val < khist->categories[c]; ++val) {
//                    printf("%u", khist->cluster_hists[m * khist->total_cat_val + khist->cat_idx[c] + val]);
//                }
//                printf("\n");
//            }
//            printf("\n");
//        }
//        fflush(stdout);

        /** end debug **/

    }

//    /** some debug **/
//    for (uint8_t m = 0; m < khist->cluster_num; ++m) {
//
//        printf("Mode %hhu:\n", m);
//        for (uint8_t c = 0; c < khist->cat_num; ++c) {
//            printf("Category %hhu: ", c);
//            for (uint8_t val = 0; val < khist->categories[c]; ++val) {
//                printf("%u", khist->cluster_hists[m * khist->total_cat_val + khist->cat_idx[c] + val]);
//            }
//            printf("\n");
//        }
//        printf("\n");
//    }
//    fflush(stdout);
//
//    /** end debug **/

    free(dlabels);
    return;

}

void train_incremental(khist_clust_t *khist, const uint8_t * const * data, size_t dsize) {

    double cc[2];
    double cost;

    printf("K-Histograms incremental training on %zu data points\n", dsize);

    if (khist->active_hists != khist->cluster_num) {
        printf("K-Histograms can not be trained incrementally, because not all histograms are initialized.\n");
        return;
    }

    cost = 0.0;
    for (size_t didx = 0; didx < dsize; ++didx) {
        assign_cluster_with_mem((const khist_clust_t *)khist, data[didx], (double *) cc);

        cost += cc[1];
        khist->cluster_totals[(uint8_t) cc[0]] += 1;

        for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
            khist->cluster_hists[((uint8_t)cc[0]) * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] += 1;
        }
    }

    printf("Completed K-Histogram clustering for %zu data points with cost: %f\n", dsize, cost);

    return;
}


void train_incremental_on_selected(
        khist_clust_t *khist,
        const uint8_t * const * data,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected)
{
    size_t didx;
    double cc[2];
    double cost;

    printf("K-Histograms incremental training on %zu data points\n", num_selected);

    if (dsize < num_selected) {
        printf("The data size (%zu) is lesser than the number of selected data points (%zu), so there must be an error\n", dsize, num_selected);
        return;
    }

    if (khist->active_hists != khist->cluster_num) {
        printf("K-Histograms can not be trained incrementally, because not all histograms are initialized.\n");
        return;
    }

    cost = 0.0;
    for (size_t sel_idx = 0; sel_idx < num_selected; ++sel_idx) {
        didx = selected_indices[sel_idx];

        assign_cluster_with_mem((const khist_clust_t *)khist, data[didx], (double *) cc);

        cost += cc[1];
        khist->cluster_totals[(uint8_t) cc[0]] += 1;

        for (uint8_t cat = 0; cat < khist->cat_num; ++cat) {
            khist->cluster_hists[((uint8_t)cc[0]) * khist->total_cat_val + khist->cat_idx[cat] + data[didx][cat]] += 1;
        }
    }

    printf("Completed K-Histogram clustering for %zu data points with cost: %f\n", num_selected, cost);

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
            cost += ((double) (khist->cluster_totals[hist])) - ((double)(khist->cluster_hists[hist*khist->total_cat_val + khist->cat_idx[cat] + data[cat]]));
        }

        cost /= (double) (khist->cluster_totals[hist]);

        if (cost < mincost || !mincost_count) {
            mincost = cost;
            memset(mincost_idx, 0, sizeof(mincost_idx));
            mincost_idx[0] = hist;
            mincost_count = 1;
        }
        else if (cost == mincost) {
            mincost_idx[mincost_count++] = hist;
        }
    }

    uint8_t mrandval = (uint8_t) random() % mincost_count;
    res[0] = (double) (mincost_count > 1 ?
//          mincost_idx[0] :
        mincost_idx[mrandval] :
        mincost_idx[0]);
    res[1] = mincost;

    return;
}

double * assign_cluster(const khist_clust_t * khist, const uint8_t * data) {
    double * res;

    if (!(res = malloc(2 * sizeof(double)))) {
        printf("Failed to allocate memory for K-Histograms clustering cluster and index\n");
        return NULL;
    }

    assign_cluster_with_mem(khist, data, res);
    return res;
}


uint8_t assign_cluster_no_cost(
        const khist_clust_t * khist,
        const uint8_t * data)
{
    double cost;
    double mincost;
    uint8_t mincost_idx[256];
    uint8_t mincost_count;


    mincost_count = 0;
    memset(mincost_idx, 0, sizeof(mincost_idx));
    for (int hist = 0; hist < khist->cluster_num; ++hist) {
        cost = 0;

        for (int cat = 0; cat < khist->cat_num; ++cat) {
            cost += ((double) (khist->cluster_totals[hist])) - ((double)(khist->cluster_hists[hist*khist->total_cat_val + khist->cat_idx[cat] + data[cat]]));
        }

        cost /= (double) (khist->cluster_totals[hist]);

        if (cost < mincost || !mincost_count) {
            mincost = cost;
            memset(mincost_idx, 0, sizeof(mincost_idx));
            mincost_idx[0] = hist;
            mincost_count = 1;
        }
        else if (cost == mincost) {
            mincost_idx[mincost_count++] = hist;
        }
    }

    uint8_t mrandval = (uint8_t) random() % mincost_count;

    return mincost_count > 1 ?
        mincost_idx[mrandval] :
        mincost_idx[0];
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
        res->cluster_totals[cl] = values[at++];
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

khist_clust_t * khist_from_file(
        FILE * file)
{
    uint8_t cluster_num;
    uint8_t cat_num;
    uint8_t * categories;
    khist_clust_t * res;

    fscanf(file, "%hhu\n", &cat_num);
    fscanf(file, "%hhu\n", &cluster_num);

    printf("Loading K-Histograms Clustering with %hhu categories and %hhu clusters\n", cat_num, cluster_num);

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for K-Histograms clustering model categories\n");
        return NULL;
    }

    for (int cat = 0; cat < cat_num; ++cat) {
        fscanf(file, "%hhu\n", &(categories[cat]));
    }

    if (!(res = create_khist((const uint8_t *) categories, cat_num, cluster_num, NULL, 0))) {
        printf("Failed to create K-Histograms clustering model\n");
        free(categories);
        return NULL;
    }

    free(categories);

    for (int cl = 0; cl < cluster_num; ++cl) {
        fscanf(file, "%u\n", &(res->cluster_totals[cl]));
    }

    for (int cl = 0; cl < cluster_num; ++cl) {
        for(int cat = 0; cat < cat_num; ++cat) {
            for (int val = 0; val < res->categories[cat]; ++val) {
                fscanf(
                        file,
                        "%u\n",
                        &(res->cluster_hists[cl * res->total_cat_val + res->cat_idx[cat] + val]));
            }
        }
    }

    return res;
}

void khist_to_file(
        khist_clust_t *khist,
        FILE * file)
{

    fprintf(file, "%hhu\n", khist->cat_num);
    fprintf(file, "%hhu\n", khist->cluster_num);

    for (int cat = 0; cat < khist->cat_num; ++cat) {
        fprintf(file, "%hhu\n", khist->categories[cat]);
    }

    for (int cl = 0; cl < khist->cluster_num; ++cl) {
        fprintf(file, "%u\n", khist->cluster_totals[cl]);
    }

    for (int cl = 0; cl < khist->cluster_num; ++cl) {
        for (int cat = 0; cat < khist->cat_num; ++cat) {
            for (int val = 0; val < khist->categories[cat]; ++val) {
                fprintf(
                        file,
                        "%u\n",
                        khist->cluster_hists[cl * khist->total_cat_val + khist->cat_idx[cat] + val]);
            }
        }
    }

    fflush(file);

    return;
}

