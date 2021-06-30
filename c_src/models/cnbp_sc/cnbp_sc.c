#include "cnbp_sc.h"

#include <stdlib.h>
#include <string.h>
#include <math.h>

static void cnbp_sc_train_single_with_scratch(
        cnbp_sc_t * cnbp_sc,
        const uint8_t * data,
        const double * label,
        double * result_scratch_space,
        uint8_t * cat_scratch_space);

cnbp_sc_t * create_cnbp_sc(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        size_t nodes_num,
        double nb_alpha,
        uint8_t use_probs)
{
    cnbp_sc_t * res;
    uint8_t * tmp_cats;

    if (!(res = malloc(sizeof(cnbp_sc_t)))) {
        printf("Failed to allocate memory for cnbp_sc classifier\n");
        return NULL;
    }

    memset(res, 0, sizeof(cnbp_sc_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for cnbp_sc classifier categories\n");
        free_cnbp_sc(res);
        return NULL;
    }

    for (size_t cn = 0; cn < cat_num; ++cn) {
        res->categories[cn] = categories[cn];
    }

    if (!(tmp_cats = malloc((cat_num + nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for temporary category array to create classifier nodes\n");
        free_cnbp_sc(res);
        return NULL;
    }

    memcpy(tmp_cats, categories, cat_num * sizeof(uint8_t));

    if (!(res->nodes = malloc(nodes_num * sizeof(cnbp_t *)))) {
        printf("Failed to allocate memory for cnbp_sc classifier nodes array\n");
        free_cnbp_sc(res);
        return NULL;
    }

    for (size_t n = 0; n < nodes_num; ++n) {

        if (n > 0) {
            tmp_cats[cat_num + n - 1] = class_num;
        }

        if (!(res->nodes[n] = create_cnbp_with_alpha(class_num, tmp_cats, cat_num + n, nb_alpha, use_probs))) {
            printf("Failed to create cnbp classifier node for cnbp_sc\n");
            free_cnbp_sc(res);
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

void cnbp_sc_train_single(
        cnbp_sc_t * cnbp_sc,
        const uint8_t * data,
        const double * label)
{
    uint8_t * cat_scratch_space;
    double * result_scratch_space;

    if (!(cat_scratch_space = malloc((cnbp_sc->cat_num + cnbp_sc->nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate category scratch space for training cnbp_sc on single data point.\n");
        return;
    }

    if (!(result_scratch_space = malloc(cnbp_sc->class_num * sizeof(double)))) {
        printf("Failed to allocate result scratch space for training cnbp_sc on single data point.\n");
        free(cat_scratch_space);
        return;
    }

    cnbp_sc_train_single_with_scratch(
        cnbp_sc,
        data,
        label,
        result_scratch_space,
        cat_scratch_space);

    free(result_scratch_space);
    free(cat_scratch_space);
}

void cnbp_sc_train_batch(
        cnbp_sc_t * cnbp_sc,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize)
{
    uint8_t * cat_scratch_space;
    double * result_scratch_space;

    if (!(cat_scratch_space = malloc((cnbp_sc->cat_num + cnbp_sc->nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate category scratch space for training cnbp_sc on batch data.\n");
        return;
    }

    if (!(result_scratch_space = malloc(cnbp_sc->class_num * sizeof(double)))) {
        printf("Failed to allocate result scratch space for training cnbp_sc on batch data.\n");
        free(cat_scratch_space);
        return;
    }

    for (size_t d_idx = 0; d_idx < dsize; ++d_idx) {
        cnbp_sc_train_single_with_scratch(
            cnbp_sc,
            datas[d_idx],
            labels[d_idx],
            result_scratch_space,
            cat_scratch_space);
    }

    free(result_scratch_space);
    free(cat_scratch_space);
}

void cnbp_sc_predict_class(
        const cnbp_sc_t * cnbp_sc,
        const uint8_t * data,
        double * labels)
{
    uint8_t * cat_scratch_space;
    uint8_t previous_result = 0;
    double max;

    if (!(cat_scratch_space = malloc((cnbp_sc->cat_num + cnbp_sc->nodes_num - 1) * sizeof(uint8_t)))) {
        printf("Failed to allocate category scratch space for predicting class by cnbp_sc.\n");
        return;
    }

    memcpy(cat_scratch_space, data, cnbp_sc->cat_num * sizeof(uint8_t));

    for (size_t n_idx = 0; n_idx < cnbp_sc->nodes_num; ++n_idx) {

        if (n_idx > 0) {
            cat_scratch_space[cnbp_sc->cat_num + n_idx - 1] = previous_result;
        }

        cnbp_predict_class(cnbp_sc->nodes[n_idx], cat_scratch_space, labels);
        max = 0.0;

        if (n_idx < cnbp_sc->nodes_num - 1) {
            for (uint8_t cl = 0; cl < cnbp_sc->class_num; ++cl) {
                if (labels[cl] > max) {
                    max = labels[cl];
                    previous_result = cl;
                }
            }
        }
    }

    free(cat_scratch_space);
}

void free_cnbp_sc(
        cnbp_sc_t * cnbp_sc)
{
    if (cnbp_sc->categories) {
        free(cnbp_sc->categories);
        cnbp_sc->categories = NULL;
    }

    if (cnbp_sc->nodes) {

        for (size_t n = 0; n < cnbp_sc->nodes_num; ++n) {
            free_cnbp(cnbp_sc->nodes[n]);
            cnbp_sc->nodes[n] = NULL;
        }

        free(cnbp_sc->nodes);
        cnbp_sc->nodes = NULL;
    }

    free(cnbp_sc);
}

cnbp_sc_t * cnbp_sc_from_file(
        FILE * file,
        uint8_t use_probs)
{
    cnbp_sc_t * res;
    uint8_t * categories;
    uint8_t * tmp_cats;
    size_t cat_num;
    uint8_t class_num;
    size_t nodes_num;
    double nb_alpha;

    fscanf(file, "%zu\n", &cat_num);

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for categories of cnbp_sc classifier being read from file.\n");
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

    if (!(res = create_cnbp_sc(
            categories,
            cat_num,
            class_num,
            nodes_num,
            nb_alpha,
            use_probs)))
    {
        printf("Failed to allocate memory for cnbp_sc classifier being read from file.\n");
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

cnbp_sc_t * cnbp_sc_from_file_with_name(
        const char * filename,
        uint8_t use_probs)
{
    FILE * file;
    cnbp_sc_t * res;

    if (!(file = fopen(filename, "r"))) {
        printf("Failed to open file %s for reading cnbp_sc model.\n", filename);
        return NULL;
    }

    if (!(res = cnbp_sc_from_file(
            file,
            use_probs)))
    {
        printf("Failed to read cnbp_sc from file: %s.\n", filename);
        fclose(file);
        return NULL;
    }

    fclose(file);
    return res;
}

void cnbp_sc_to_file(
        const cnbp_sc_t * cnbp_sc,
        FILE * file)
{
    fprintf(file, "%zu\n", cnbp_sc->cat_num);

    for (size_t cat = 0; cat < cnbp_sc->cat_num; ++cat) {
        fprintf(file, "%hhu\n", cnbp_sc->categories[cat]);
    }

    fprintf(file, "%hhu\n", cnbp_sc->class_num);
    fprintf(file, "%zu\n", cnbp_sc->nodes_num);
    fprintf(file, "%lf\n", cnbp_sc->nb_alpha);

    for (size_t n = 0; n < cnbp_sc->nodes_num; ++n) {
        cnbp_to_file(cnbp_sc->nodes[n], file);
    }

    return;
}

void cnbp_sc_to_file_with_name(
        const cnbp_sc_t * cnbp_sc,
        const char * filename)
{
    FILE * file;

    if (!(file = fopen(filename, "w"))) {
        printf("Failed to open file %s for writing cnbp_sc model.\n", filename);
        return;
    }

    cnbp_sc_to_file(
            cnbp_sc,
            file);
    fclose(file);
    return;
}

static void cnbp_sc_train_single_with_scratch(
        cnbp_sc_t * cnbp_sc,
        const uint8_t * data,
        const double * label,
        double * result_scratch_space,
        uint8_t * cat_scratch_space)
{
    uint8_t previous_result = 0;
    memcpy(cat_scratch_space, data, cnbp_sc->cat_num);
    double max;
    size_t to_train;

    to_train = (size_t) rand() % cnbp_sc->nodes_num;

    if (to_train == 0) {
        cnbp_train_single(cnbp_sc->nodes[0], data, label);
    }
    else {
        for (size_t n_idx = 0; n_idx < to_train; ++n_idx) {

            cnbp_predict_class(cnbp_sc->nodes[n_idx], (const uint8_t *) cat_scratch_space, result_scratch_space);

            max = 0.0;

            for (uint8_t cl = 0; cl < cnbp_sc->class_num; ++cl) {
                if (result_scratch_space[cl] > max) {
                    max = result_scratch_space[cl];
                    previous_result = cl;
                }
            }

            cat_scratch_space[cnbp_sc->cat_num + n_idx - 1] = previous_result;
        }

        cnbp_train_single(cnbp_sc->nodes[to_train], cat_scratch_space, label);
    }
}
