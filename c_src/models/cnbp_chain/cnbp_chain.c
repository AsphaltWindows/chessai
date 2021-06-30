#include "cnbp_chain.h"

#include <stdlib.h>
#include <string.h>
#include <math.h>

static void cnbp_chain_train_single_with_scratch(
        cnbp_chain_t * cnbp_chain,
        const uint8_t * data,
        const double * label,
        double * result_scratch_space,
        uint8_t * cat_scratch_space);

cnbp_chain_t * create_cnbp_chain(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        size_t nodes_num,
        double nb_alpha,
        uint8_t use_probs)
{
    cnbp_chain_t * res;
    uint8_t * tmp_cats;

    if (!(res = malloc(sizeof(cnbp_chain_t)))) {
        printf("Failed to allocate memory for cnbp_chain classifier\n");
        return NULL;
    }

    memset(res, 0, sizeof(cnbp_chain_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for cnbp_chain classifier categories\n");
        free_cnbp_chain(res);
        return NULL;
    }

    for (size_t cn = 0; cn < cat_num; ++cn) {
        res->categories[cn] = categories[cn];
    }

    if (!(tmp_cats = malloc((cat_num + nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for temporary category array to create classifier nodes\n");
        free_cnbp_chain(res);
        return NULL;
    }

    memcpy(tmp_cats, categories, cat_num * sizeof(uint8_t));

    if (!(res->nodes = malloc(nodes_num * sizeof(cnbp_t *)))) {
        printf("Failed to allocate memory for cnbp_chain classifier nodes array\n");
        free_cnbp_chain(res);
        return NULL;
    }

    for (size_t n = 0; n < nodes_num; ++n) {

        if (n > 0) {
            tmp_cats[cat_num + n - 1] = class_num;
        }

        if (!(res->nodes[n] = create_cnbp_with_alpha(class_num, tmp_cats, cat_num + n, nb_alpha, use_probs))) {
            printf("Failed to create cnbp classifier node for cnbp_chain\n");
            free_cnbp_chain(res);
            return NULL;
        }

    }

    free(tmp_cats);

    res->cat_num = cat_num;
    res->nodes_num = nodes_num;
    res->class_num = class_num;
    res->nb_alpha = nb_alpha;
    res->use_probs = use_probs;

    return res;
}

void cnbp_chain_train_single(
        cnbp_chain_t * cnbp_chain,
        const uint8_t * data,
        const double * label)
{
    uint8_t * cat_scratch_space;
    double * result_scratch_space;

    if (!(cat_scratch_space = malloc((cnbp_chain->cat_num + cnbp_chain->nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate category scratch space for training cnbp_chain on single data point.\n");
        return;
    }

    if (!(result_scratch_space = malloc(cnbp_chain->class_num * sizeof(double)))) {
        printf("Failed to allocate result scratch space for training cnbp_chain on single data point.\n");
        free(cat_scratch_space);
        return;
    }

    cnbp_chain_train_single_with_scratch(
        cnbp_chain,
        data,
        label,
        result_scratch_space,
        cat_scratch_space);

    free(result_scratch_space);
    free(cat_scratch_space);
}

void cnbp_chain_train_batch(
        cnbp_chain_t * cnbp_chain,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize)
{
    uint8_t * cat_scratch_space;
    double * result_scratch_space;

    if (!(cat_scratch_space = malloc((cnbp_chain->cat_num + cnbp_chain->nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate category scratch space for training cnbp_chain on batch data.\n");
        return;
    }

    if (!(result_scratch_space = malloc(cnbp_chain->class_num * sizeof(double)))) {
        printf("Failed to allocate result scratch space for training cnbp_chain on batch data.\n");
        free(cat_scratch_space);
        return;
    }

    for (size_t d_idx = 0; d_idx < dsize; ++d_idx) {
        cnbp_chain_train_single_with_scratch(
            cnbp_chain,
            datas[d_idx],
            labels[d_idx],
            result_scratch_space,
            cat_scratch_space);
    }

    free(result_scratch_space);
    free(cat_scratch_space);
}

void cnbp_chain_predict_class(
        const cnbp_chain_t * cnbp_chain,
        const uint8_t * data,
        double * labels)
{
    uint8_t * cat_scratch_space;
    uint8_t previous_result = 0;
    double max;

    if (!(cat_scratch_space = malloc((cnbp_chain->cat_num + cnbp_chain->nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate category scratch space for predicting class by cnbp_chain.\n");
        return;
    }

    memcpy(cat_scratch_space, data, cnbp_chain->cat_num * sizeof(uint8_t));

    for (size_t n_idx = 0; n_idx < cnbp_chain->nodes_num; ++n_idx) {

        if (n_idx > 0) {
            cat_scratch_space[cnbp_chain->cat_num + n_idx - 1] = previous_result;
        }

        cnbp_predict_class(cnbp_chain->nodes[n_idx], cat_scratch_space, labels);
        max = 0.0;

        if (n_idx < cnbp_chain->nodes_num - 1) {
            for (uint8_t cl = 0; cl < cnbp_chain->class_num; ++cl) {
                if (labels[cl] > max) {
                    max = labels[cl];
                    previous_result = cl;
                }
            }
        }
    }

    free(cat_scratch_space);
}

void free_cnbp_chain(
        cnbp_chain_t * cnbp_chain)
{
    if (cnbp_chain) {

        if (cnbp_chain->categories) {
            free(cnbp_chain->categories);
            cnbp_chain->categories = NULL;
        }

        if (cnbp_chain->nodes) {

            for (size_t n = 0; n < cnbp_chain->nodes_num; ++n) {
                free_cnbp(cnbp_chain->nodes[n]);
                cnbp_chain->nodes[n] = NULL;
            }

            free(cnbp_chain->nodes);
            cnbp_chain->nodes = NULL;
        }

        free(cnbp_chain);
    }
}

cnbp_chain_t * cnbp_chain_from_file(
        FILE * file,
        uint8_t use_probs)
{
    cnbp_chain_t * res;
    uint8_t * categories;
    uint8_t * tmp_cats;
    size_t cat_num;
    uint8_t class_num;
    size_t nodes_num;
    double nb_alpha;

    fscanf(file, "%zu\n", &cat_num);

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for categories of cnbp_chain classifier being read from file.\n");
        return NULL;
    }

    for (size_t cat = 0; cat < cat_num; ++cat) {
        fscanf(file, "%hhu\n", &(categories[cat]));
    }

    fscanf(file, "%hhu\n", &class_num);
    fscanf(file, "%zu\n", &nodes_num);
    fscanf(file, "%lf\n", &nb_alpha);

    if (!(tmp_cats = malloc((cat_num + nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for temporary category array to create classifier nodes\n");
        return NULL;
    }

    memcpy(tmp_cats, categories, cat_num * sizeof(uint8_t));

    if (!(res = create_cnbp_chain(
            categories,
            cat_num,
            class_num,
            nodes_num,
            nb_alpha,
            use_probs)))
    {
        printf("Failed to allocate memory for cnbp_chain classifier being read from file.\n");
        free(categories);
        free(tmp_cats);
        return NULL;
    }

    for (size_t n = 0; n < res->nodes_num; ++n) {
        free_cnbp(res->nodes[n]);

        if (n > 0) {
            tmp_cats[cat_num + n - 1] = class_num;
        }

        res->nodes[n] = cnbp_from_file_with_params(
            file,
            class_num,
            tmp_cats,
            cat_num + n,
            nb_alpha,
            use_probs);
    }

    free(tmp_cats);
    return res;
}

cnbp_chain_t * cnbp_chain_from_file_with_name(
        const char * filename,
        uint8_t use_probs)
{
    FILE * file;
    cnbp_chain_t * res;

    if (!(file = fopen(filename, "r"))) {
        printf("Failed to open file %s for reading cnbp_chain model.\n", filename);
        return NULL;
    }

    if (!(res = cnbp_chain_from_file(
            file,
            use_probs)))
    {
        printf("Failed to read cnbp_chain from file: %s.\n", filename);
        fclose(file);
        return NULL;
    }

    fclose(file);
    return res;
}

void cnbp_chain_to_file(
        const cnbp_chain_t * cnbp_chain,
        FILE * file)
{
    fprintf(file, "%zu\n", cnbp_chain->cat_num);

    for (size_t cat = 0; cat < cnbp_chain->cat_num; ++cat) {
        fprintf(file, "%hhu\n", cnbp_chain->categories[cat]);
    }

    fprintf(file, "%hhu\n", cnbp_chain->class_num);
    fprintf(file, "%zu\n", cnbp_chain->nodes_num);
    fprintf(file, "%lf\n", cnbp_chain->nb_alpha);

    for (size_t n = 0; n < cnbp_chain->nodes_num; ++n) {
        cnbp_to_file(cnbp_chain->nodes[n], file);
    }

    return;
}

void cnbp_chain_to_file_with_name(
        const cnbp_chain_t * cnbp_chain,
        const char * filename)
{
    FILE * file;

    if (!(file = fopen(filename, "w"))) {
        printf("Failed to open file %s for writing cnbp_chain model.\n", filename);
        return;
    }

    cnbp_chain_to_file(
            cnbp_chain,
            file);
    fclose(file);
    return;
}

static void cnbp_chain_train_single_with_scratch(
        cnbp_chain_t * cnbp_chain,
        const uint8_t * data,
        const double * label,
        double * result_scratch_space,
        uint8_t * cat_scratch_space)
{
    uint8_t previous_result = 0;
    memcpy(cat_scratch_space, data, cnbp_chain->cat_num);
    double max;

    for (size_t n_idx = 0; n_idx < cnbp_chain->nodes_num; ++n_idx) {

        if (n_idx > 0) {
            cat_scratch_space[cnbp_chain->cat_num + n_idx - 1] = previous_result;
        }

        cnbp_train_single(cnbp_chain->nodes[n_idx], (const uint8_t *) cat_scratch_space, label);
        cnbp_predict_class(cnbp_chain->nodes[n_idx], cat_scratch_space, result_scratch_space);

        max = 0.0;

        for (uint8_t cl = 0; cl < cnbp_chain->class_num; ++cl) {
            if (result_scratch_space[cl] > max) {
                max = result_scratch_space[cl];
                previous_result = cl;
            }
        }
    }
}
