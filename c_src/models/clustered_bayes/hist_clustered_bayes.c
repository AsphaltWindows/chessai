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
        uint8_t node_type,
        cnb_clas_t * parent_classifier);

static void free_hcb_node(
        hcb_node_t *node);

static void train_model_node(
        hcb_node_t * node,
        const uint8_t * const * data,
        const uint8_t * labels,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected);

static void hcb_node_to_file(
        const hcb_node_t * node,
        FILE * file);

static hcb_node_t * hcb_node_from_file(
        FILE * file);


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
                    LEAF,
                    NULL))) {
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

void train_model(
        hcb_clas_t *hcb,
        const uint8_t * const * data,
        const uint8_t * labels,
        size_t dsize)
{
    size_t * selected_dummy;

    if (!(selected_dummy = malloc(dsize * sizeof(size_t)))) {
        printf("Failed to allocate memory for array of dummy selected indices to begin initial model training\n");
        return;
    }

    for (unsigned int i = 0; i < dsize; ++i) {
        selected_dummy[i] = i;
    }

    train_model_node(
            hcb->root,
            data,
            labels,
            dsize,
            (const size_t *) selected_dummy,
            dsize);

    free(selected_dummy);
}

double * hcb_predict_class(const hcb_clas_t * hcb, const uint8_t * data);

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
        uint32_t hierarchy_size,
        uint8_t local_hierarchy,
        double alpha,
        uint32_t limit,
        uint8_t forget_factor,
        uint8_t node_type,
        cnb_clas_t * parent_classifier)
{

    hcb_node_t * res;

    if (parent_classifier && node_type != LEAF) {
        printf("A parent classifier was passed to node creation, but the desired node_type is not LEAF\n");
        return NULL;
    }

    if (!(res = malloc(sizeof(hcb_node_t)))) {
        printf("Failed to allocate memory for hcb node\n");
        return NULL;
    }

    memset(res, 0, sizeof(hcb_clas_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for hcb node categories\n");
        free_hcb_node(res);
        return NULL;
    }

    for (unsigned int cn = 0; cn < cat_num; ++cn) {
        res->categories[cn] = cats[cn];
    }

    if (!(res->hierarchy = malloc((hierarchy_size + 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for hcb node hierarchy\n");
        free_hcb_node(res);
        return NULL;
    }

    if (hierarchy_size) {
        memcpy(res->hierarchy, parent_hierarchy, (size_t) hierarchy_size * sizeof(uint8_t));
    }

    res->hierarchy[hierarchy_size] = local_hierarchy;
    res->hierarchy_size = hierarchy_size + 1;
    res->class_num = class_num;
    res->cat_num = cat_num;
    res->cluster_num = cluster_num;
    res->limit = limit;
    res->alpha = alpha;
    res->forget_factor = forget_factor;
    res->node_type = node_type;
    res->num_trained = 0;

    if (res->node_type == LEAF) {
        
        if (!(res->classifier = create_cnb_with_alpha(class_num, cats, cat_num, alpha))) {
            printf("Failed to create Categorical Naive Bayes classifier for hcb classifier leaf node\n");
            free_hcb_node(res);
            return NULL;
        }

        if (parent_classifier) {

            for (unsigned int i = 0; i < class_num; ++i) {

                res->classifier->class_totals[i] = parent_classifier->class_totals[i] / forget_factor;

            }

            for (unsigned int i = 0; i < cat_num; ++i) {

                res->classifier->class_cat_idx[i] = parent_classifier->class_cat_idx[i] / forget_factor;
                
            }

            for (unsigned int i = 0; i < (parent_classifier->total_cat_vals * class_num); ++i) {

                res->classifier->class_cat_totals[i] = parent_classifier->class_cat_totals[i] / forget_factor;

            }

            recalculate_probabilities(res->classifier);

        }

        res->clustering = NULL;
    }

    else if (res->node_type == BRANCH) {

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
                            LEAF,
                            NULL)))
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

        for (uint8_t i = 0; i < node->cluster_num; ++i) {
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
        const size_t * selected_indices,
        size_t num_selected)
{
    khist_clust_t * clustering;
    size_t * label_counts;
    size_t didx;
    size_t ** split_data_indices;
    uint8_t tmp_label;

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

        node->num_trained += num_selected;

        if (node->num_trained > node->limit) {

            if (!(node->clustering = create_khist(
                           node->categories,
                           node->cat_num,
                           node->cluster_num,
                           NULL,
                           0))) {
                printf("Failed to create new k-histograms clustering\n");
                return;
            }

            node->node_type = BRANCH;

            for (uint8_t cidx = 0; cidx < node->cluster_num; ++cidx) {
                
                if (!( node->children[cidx] = create_hcb_node(
                    node->class_num,
                    node->categories,
                    node->cat_num,
                    node->cluster_num,
                    node->hierarchy,
                    node->hierarchy_size,
                    cidx,
                    node->classifier->alpha,
                    node->limit,
                    node->forget_factor,
                    LEAF,
                    node->classifier))) {
                    printf("Failed to create child node for new transformed BRANCH node\n");

                    for (uint8_t cidx2 = 0; cidx2 < cidx; ++cidx2) {
                        free_hcb_node(
                                node->children[cidx2]);
                        node->children[cidx2] = NULL;
                    }

                    return;
                }

            }

            free_cnb(node->classifier);
            node->classifier = NULL;

            train_model_node(
                node,
                data,
                labels,
                dsize,
                selected_indices,
                num_selected);

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

void hcb_model_to_file(
        const hcb_clas_t * hcb,
        const char * filename)
{
    FILE * file;

    if (!(file = fopen(filename, "w"))) {
        printf("Failed to open file %s for writing Histogram Clustered Bayes model.\n", filename);
        return;
    }

    fprintf(file, "%hhu\n", hcb->class_num);
    fprintf(file, "%hhu\n", hcb->cat_num);

    for (uint8_t cat = 0; cat < hcb->cat_num; ++cat) {
        fprintf(file, "%hhu\n", hcb->categories[cat]);
    }

    fprintf(file, "%hhu\n", hcb->alpha);
    fprintf(file, "%u\n", hcb->forget_factor);
    fprintf(file, "%u\n", hcb->limit);
    fprintf(file, "%hhu\n", hcb->cluster_num);

    hcb_node_to_file(
            hcb->root,
            file);

    fflush(file);
    fclose(file);

    return;
}

static void hcb_node_to_file(
        const hcb_node_t * node,
        FILE * file)
{
    fprintf(file, "%hhu\n", node->class_num);
    fprintf(file, "%hhu\n", node->cat_num);

    for (uint8_t cat = 0; cat < node->cat_num; ++cat) {
        fprintf(file, "%hhu\n", node->categories[cat]);
    }

    fprintf(file, "%u\n", node->hierarchy_size);

    for (uint8_t h = 0; h < node->hierarchy_size; ++h) {
        fprintf(file, "%hhu\n", node->hierarchy[h]);
    }

    fprintf(file, "%hhu\n", node->node_type);
    fprintf(file, "%hhu\n", node->alpha);
    fprintf(file, "%hhu\n", node->forget_factor);
    fprintf(file, "%u\n", node->limit);
    fprintf(file, "%u\n", node->num_trained);

    if (node->node_type == LEAF) {
        cnb_to_file(
                node->classifier,
                file);
    }

    fprintf(file, "%hhu\n", node->cluster_num);

    if (node->node_type == BRANCH) {
        khist_to_file(
                node->clustering,
                file);

        for (uint8_t clust = 0; clust < node->cluster_num; ++clust) {
            hcb_node_to_file(
                    node->children[clust],
                    file);
        }
    }

    fflush(file);

    return;
}

hcb_clas_t * hcb_model_from_file(
        const char * filename)
{
    FILE * file;
    hcb_clas_t * res;

    if (!(file = fopen(filename, "r"))) {
        printf("Failed to open file %s for reading Histogram Clustered Bayes model.\n", filename);
        return NULL;
    }

    if (!(res = malloc(sizeof(hcb_clas_t)))) {
        printf("Failed to allocate memory for Histogram Clustered Bayes model.\n");
        return NULL;
    }

    memset(res, 0, sizeof(hcb_clas_t));

    fscanf(file, "%hhu\n", &(res->class_num));
    fscanf(file, "%hhu\n", &(res->cat_num));

    if (!(res->categories = malloc(sizeof(uint8_t) * res->cat_num))) {
        printf("Failed to allocate memory for categories array of Histogram Clustered Bayes model.\n");
        free_hcb(res);
        return NULL;
    }

    for (uint8_t cat = 0; cat < res->cat_num; ++cat) {
        fscanf(file, "%hhu\n", &(res->categories[cat]));
    }

    fscanf(file, "%hhu\n", &(res->alpha));
    fscanf(file, "%hhu\n", &(res->forget_factor));
    fscanf(file, "%u\n", &(res->limit));
    fscanf(file, "%hhu\n", &(res->cluster_num));

    if (!(res->root = hcb_node_from_file(
                    file))) {
        printf("Failed to deserialize Histogram Clustered Bayes root node.\n");
        free_hcb(res);
        return NULL;
    }

    fclose(file);

    return res;
}

static hcb_node_t * hcb_node_from_file(
        FILE * file)
{

    hcb_node_t * res;

    if (!(res = malloc(sizeof(hcb_node_t)))) {
        printf("Failed to allocate memory for Histogram Clustered Bayes node.\n");
        return NULL;
    }

    memset(res, 0, sizeof(hcb_node_t));

    fscanf(file, "%hhu\n", &(res->class_num));
    fscanf(file, "%hhu\n", &(res->cat_num));

    if (!(res->categories = malloc(sizeof(uint8_t) * res->cat_num))) {
        printf("Failed to allocate memory for categories array of Histogram Clustered Bayes node.\n");
        free_hcb_node(res);
        return NULL;
    }
    
    memset(res->categories, 0, sizeof(uint8_t) * res->cat_num);

    for (uint8_t cat = 0; cat < res->cat_num; ++cat) {
        fscanf(file, "%hhu\n", &(res->categories[cat]));
    }

    fscanf(file, "%u\n", &(res->hierarchy_size));

    if (!(res->hierarchy = malloc(sizeof(uint8_t) * res->hierarchy_size))) {
        printf("Failed to allocate memory for hierarchy array of Histogram Clustered Bayes node.\n");
        free_hcb_node(res);
        return NULL;
    }
    
    memset(res->hierarchy, 0, sizeof(uint8_t) * res->hierarchy_size);

    for (uint8_t h = 0; h < res->hierarchy_size; ++h) {
        fscanf(file, "%hhu\n", &(res->hierarchy[h]));
    }

    fscanf(file, "%hhu\n", &(res->node_type));
    fscanf(file, "%hhu\n", &(res->alpha));
    fscanf(file, "%hhu\n", &(res->forget_factor));
    fscanf(file, "%u\n", &(res->limit));
    fscanf(file, "%u\n", &(res->num_trained));

    if (res->node_type == LEAF) {
        if (!(res->classifier = cnb_from_file(
                        file))) {
            printf("Failed to deserialize Categorical Naive Bayes classifier from file.\n");
            free_hcb_node(res);
            return NULL;
        }
    }

    fscanf(file, "%hhu\n", &(res->cluster_num));
    
    if (res->node_type == BRANCH) {
        if (!(res->clustering = khist_from_file(
                        file))) {
            printf("Failed to deserialize Histogram Clustering from file.\n");
            free_hcb_node(res);
            return NULL;
        }

        if (!(res->children = malloc(res->cluster_num * sizeof(hcb_node_t *)))) {
            printf("Failed to allocate array for children nodes of Histogram Clustered Bayes node.\n");
            free_hcb_node(res);
            return NULL;
        }

        memset(res->children, 0, res->cluster_num * sizeof(hcb_node_t *));

        for (uint8_t clust = 0; clust < res->cluster_num; ++clust) {

            if (!(res->children[clust] = hcb_node_from_file(
                            file))) {
                printf("Failed to deserialize child Histogram Clustered Bayes node.\n");
                free_hcb_node(res);
                return NULL;
            }
        }
    }

    return res;
}

