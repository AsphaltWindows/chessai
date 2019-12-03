#include "hist_clustered_bayes.h"

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>

#include "../naive_bayes/categorical_naive_bayes.h"
#include "../k_histograms/k_hist.h"

static hcb_node_t * create_hcb_node(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        uint8_t cluster_num,
        const uint8_t * parent_hierarchy,
        uint32_t hierachy_size,
        uint8_t local_hierarchy,
        double alpha,
        uint32_t limit,
        uint8_t forget_factor,
        uint8_t node_type);

static void free_hcb_node(hcb_node_t *node);

static void train_model_node(
        hcb_node_t * node,
        const uint8_t * const * data,
        const uint8_t * labels,
        size_t dsize);

hcb_clas_t * create_hcb(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        uint8_t cluster_num,
        double alpha,
        uint32_t limit,
        uint8_t forget_factor)
{
    hcb_clas_t *res;

    if (!(res = malloc(sizeof(hcb_clas_t)))) {
        printf("Failed to allocate memory for hcb classifier\n");
        return NULL;
    }

    memset(res, 0, sizeof(hcb_clas_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for hcb classifier categories\n");
        free_hcb(res);
        return NULL;
    }

    for (unsigned int cn = 0; cn < cat_num; ++cn) {
        res->categories[cn] = cats[cn];
    }

    if (!(res->root = create_hcb_node(
                    class_num,
                    cats,
                    cat_num,
                    cluster_num,
                    NULL,
                    0,
                    0,
                    alpha,
                    limit,
                    forget_factor,
                    LEAF))) {
        printf("Failed to create root node for hcb classifier\n");
        free_hcb(res);
        return NULL;
    }

    res->class_num = class_num;
    res->cat_num = cat_num;
    res->cluster_num = cluster_num;
    res->alpha = alpha;
    res->limit = limit;
    res->forget_factor = forget_factor;

    return res;
}

void train_model(hcb_clas_t *hcb, const uint8_t * const * data, const uint8_t * labels, size_t dsize) {
    train_model_node(hcb->root, data, labels, dsize);
}

double * predict_class(const hcb_clas_t * hcb, const uint8_t * data);

hcb_clas_t * hcb_model_from_vals(const uint32_t * values, size_t num_values);

uint32_t * hcb_model_to_vals(const hcb_clas_t * hcb);

void free_hcb(hcb_clas_t *hcb) {
    
    if (!hcb) {
        return;
    }

    if (hcb->categories) {
        free(hcb->categories);
        hcb->categories = NULL;
    }

    if (hcb->root) {
        free_hcb_node(hcb->root);
        hcb->root = NULL;
    }

    free(hcb);
}

static hcb_node_t * create_hcb_node(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        uint8_t cluster_num,
        const uint8_t * parent_hierarchy,
        uint32_t hierachy_size,
        uint8_t local_hierarchy,
        double alpha,
        uint32_t limit,
        uint8_t forget_factor,
        uint8_t node_type)
{

    hcb_node_t * res;

    if (!(res = malloc(sizeof(hcb_node_t)))) {
        printf("Failed to allocate memory for hcb node\n");
        return NULL;
    }

    memset(res, 0, sizeof(hcb_clas_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for hcb node categories\n");
        free_hcb(res);
        return NULL;
    }

    for (unsigned int cn = 0; cn < cat_num; ++cn) {
        res->categories[cn] = cats[cn];
    }

    if (!(res->hierarchy = malloc((hierarchy_size + 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for hcb node hierarchy\n");
        free_hcb(res);
        return NULL;
    }

    if (hierarchy_size) {
        memcpy(res->hierarchy, parent_hierarchy, (size_t) hierarchy_size * sizeof(uint8_t));
    }

    res->hierarchy[hierarchy_size] = local_hierarachy;
    res->hierarchy_size = hierarchy_size + 1;
    res->class_num = class_num;
    res->cat_num = cat_num;
    res->cluster_num = cluster_num;
    res->limit = limit;
    res->forget_factor = forget_factor;
    res->node_type = node_type;
    res->num_trained = 0;

    if (res->node_type == LEAF) {
        
        if (!(res->classifier = create_cnb_with_alpha(class_num, cats, cat_num, alpha))) {
            printf("Failed to create Categorical Naive Bayes classifier for hcb classifier leaf node\n");
            free_hcb_node(res);
            return NULL;
        }

        res->clustering = NULL;
    }

    else if(res->node_type = BRANCH) {

        if (!(res->clustering = create_khist(cats, cat_num, cluster_num, NULL, 0))) {
            printf("Failed to create K-Histograms clustering for hcb classifier branch node\n");
            free_hcb_node(res);
            return NULL;
        }

        if (!(res->children = malloc(cluster_num * sizeof(hcb_node_t *)))) {
            printf("Failed to allocate memory for HCB node children array\n");
            free_hcb_node(res);
        }

        memset(res->children, 0, cluster_num * sizeof(hcb_node_t *));

        for (uint8_t i = 0; i < cluster_num; ++i) {

            if (!(res->children[i] = create_hcb_node(
                            class_num,
                            cats,
                            cat_num,
                            cluster_num,
                            res->hierarchy,
                            res->hierarchy_size,
                            i,
                            alpha,
                            limit,
                            forget_factor,
                            LEAF)))
            {
                printf("Failed to create HCB leaf node for HCB branch node of HCB classifier\n");
                free_hcb_node(res);
            }

        }
    }

    return res;
}


static void free_hcb_node(hcb_node_t *node) {

    if (!node) {
        return;
    }

    if (node->categories) {
        free(node->categories);
        node->categories = NULL;
    }

    if (node->hierarchy) {
        free(node->hierarchy);
        node->hierarchy = NULL;
    }

    if (node->classifier) {
        free_cnb(node->classifier);
        node->classifier = NULL;
    }

    if (node->clustering) {
        free_khist(node->clustering);
        node->clustering = NULL;
    }

    if (node->children) {

        for (uint8_t i = 0; i < cluster_num; ++i) {
            if (node->children[i]) {
                free_hcb_node(node->children[i]);
                node->children[i] = NULL;
            }
        }

        free(node->children);
        node->children = NULL;
    }

    free(node);
}


static void train_model_node(
        hcb_node_t * node,
        const uint8_t * const * data,
        const uint8_t * labels,
        size_t dsize,
        const uint8_t * selected_indices,
        size_t num_selected)
{
    khist_clust_t * clustering;
    size_t * label_counts;
    uint8_t tmp_label;
    size_t didx;
    size_t ** split_data_indices;

    if (dsize < num_selected) {
        printf("The data size (%zu) is lesser than the number of selected data points (%zu), so there must be an error\n", dsize, num_selected);
        return;
    }

    if (node->node_type == LEAF) {

        train_batch_on_selected(
                node->classifier,
                data,
                labels,
                dsize,
                selected_indices,
                num_selected);

        if (node->classifier->total_cat_vals > node->limit) {
            // Here we split the node
        }
    }
    else if (node->node_type == BRANCH) {

        if (!(label_counts = malloc(node->cluster_num * sizeof(size_t)))) {
            printf("Failed to allocate memory for array to store counts of differently labeled datapoints\n");
            return;
        }

        memset(label_counts, 0, node->cluster_num * sizeof(size_t));

        if (!(split_data_indices = malloc(node->cluster_num * sizeof(size_t *)))) {
            printf("Failed to allocate memory for array for points to arrays of cluster labels of training data\n");
            free(label_counts);
            return;
        }

        memset(split_data_indices, 0, node->cluster_num * sizeof(size_t *));

        for (unsigned int i = 0; i < node->cluster_num; ++i) {
            
            if (!(split_data_indices[i] = malloc(num_selected * sizeof(size_t)))) {
                printf("Failed to allocate memory for array to store cluster labels of training data\n");

                for (unsigned int j = 0; j < i; ++j) {
                    free(split_data_indices[j]);
                }

                free(split_data_indices);
                free(label_counts);
                return;
            }

            memset(split_data_indices[i], 0, num_selected * sizeof(size_t));
        }

        for (unsigned int sel_idx = 0; sel_idx < num_selected; ++sel_idx) {
            didx = selected_indices[didx];
            clustering = node->clustering;
            tmp_label = assign_cluster_no_cost(
                    clustering,
                    data[didx]);
            split_data_indices[tmp_label][label_counts[tmp_label]++] = didx;
        }

        train_incremental_on_selected(
                clustering,
                data,
                dsize,
                selected_indices,
                num_selected);

        for (unsigned int cidx = 0; cidx < node->cluster_num; ++cidx) {

            train_model_node(
                node->children[cidx],
                data,
                labels,
                dsize,
                split_data_indices[cidx],
                label_counts[cidx]);

            free(split_data_indices[cidx]);
        }

        free(split_data_indices);
        free(label_counts);
    }
    else {
        printf("Encountered an invalid node type.\n");
    }

    return;
}
