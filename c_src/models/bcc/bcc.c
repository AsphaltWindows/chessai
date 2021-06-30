#include "bcc.h"

#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

static void bcc_train_single_with_scratch(
        bcc_t * bcc,
        const uint8_t * data,
        const double * label,
        double * root_scratch_space);

bcc_t * create_bcc(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        double nb_alpha,
        uint8_t use_probs)
{
    bcc_t * res;

    if (!(res = malloc(sizeof(bcc_t)))) {
        printf("Failed to allocate memory for bcc classifier\n");
        return NULL;
    }

    memset(res, 0, sizeof(bcc_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for bcc classifier categories\n");
        free_bcc(res);
        return NULL;
    }

    for (size_t cn = 0; cn < cat_num; ++cn) {
        res->categories[cn] = categories[cn];
    }

    if (!(res->root = create_cnbp_with_alpha(class_num, categories, cat_num, nb_alpha, use_probs))) {
        printf("Failed to create root cnbp classifier node for bcc\n");
        free_bcc(res);
        return NULL;
    }

    if (!(res->children = malloc(class_num * sizeof(cnbp_t *)))) {
        printf("Failed to allocate memory for children cnbp classifiers for bcc\n");
        free_bcc(res);
        return NULL;
    }

    memset(res->children, 0, class_num * sizeof(cnbp_t *));

    for (uint8_t cl = 0; cl < class_num; ++cl) {
        if (!(res->children[cl] = create_cnbp_with_alpha(class_num, categories, cat_num, nb_alpha, use_probs))) {
            printf("Failed to create child cnbp for bcc clasifier\n");
            free_bcc(res);
            return NULL;
        }
    }

    res->cat_num = cat_num;
    res->class_num = class_num;
    res->nb_alpha = nb_alpha;
    res->use_probs = use_probs;

    return res;
}

void bcc_train_single(
        bcc_t * bcc,
        const uint8_t * data,
        const double * label)
{
    double * root_scratch_space;

    if (!(root_scratch_space = malloc(bcc->class_num * sizeof(double)))) {
        printf("Failed to allocate memory for root scratch space for training.\n");
        return;
    }

    memset(root_scratch_space, 0, bcc->class_num * sizeof(double));

    bcc_train_single_with_scratch(bcc, data, label, root_scratch_space);
}

void bcc_train_batch(
        bcc_t * bcc,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize)
{
    double * root_scratch_space;

    if (!(root_scratch_space = malloc(bcc->class_num * sizeof(double)))) {
        printf("Failed to allocate memory for root scratch space for training.\n");
        return;
    }

    memset(root_scratch_space, 0, bcc->class_num * sizeof(double));

    for (size_t d_idx = 0; d_idx < dsize; ++d_idx) {
        bcc_train_single_with_scratch(
                bcc,
                datas[d_idx],
                labels[d_idx],
                root_scratch_space);
        memset(root_scratch_space, 0, bcc->class_num * sizeof(double));
    }
}

void bcc_predict_class(
        const bcc_t * bcc,
        const uint8_t * data,
        double * labels)
{
    double max = 0.0;
    uint8_t root_result = 0;

    cnbp_predict_class(bcc->root, data, labels);

    for (uint8_t cl = 0; cl < bcc->class_num; ++cl) {

        if (labels[cl] > max) {
            max = labels[cl];
            root_result = cl;
        }
    }

    cnbp_predict_class(bcc->children[root_result], data, labels);
}


void free_bcc(
        bcc_t * bcc)
{
    if (bcc) {

        if (bcc->root) {
            free_cnbp(bcc->root);
            bcc->root = NULL;
        }

        if (bcc->categories) {
            free(bcc->categories);
            bcc->categories = NULL;
        }

        if (bcc->children) {

            for (size_t cl = 0; cl < bcc->class_num; ++cl) {

                if (bcc->children[cl]) {
                    free_cnbp(bcc->children[cl]);
                    bcc->children[cl] = NULL;
                }

            }

            free(bcc->children);
            bcc->children = NULL;
        }

        free(bcc);
    }

}

bcc_t * bcc_from_file(
        FILE * file,
        uint8_t use_probs)
{
    bcc_t * res;
    uint8_t * categories;
    size_t cat_num;
    uint8_t class_num;
    double nb_alpha;

    fscanf(file, "%zu\n", &cat_num);

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for categories of bcc classifier being read from file.\n");
        return NULL;
    }

    for (size_t cat = 0; cat < cat_num; ++cat) {
        fscanf(file, "%hhu\n", &(categories[cat]));
    }

    fscanf(file, "%hhu\n", &class_num);
    fscanf(file, "%lf\n", &nb_alpha);

    if (!(res = create_bcc(
            categories,
            cat_num,
            class_num,
            nb_alpha,
            use_probs)))
    {
        printf("Failed to allocate memory for bcc classifier being read from file.\n");
        free(categories);
        return NULL;
    }

    if (!(res->root = cnbp_from_file_with_params(
            file,
            class_num,
            categories,
            cat_num,
            nb_alpha,
            use_probs))) {
        printf("Failed to read root cnbp from file for bcc.\n");
        free_bcc(res);
        return NULL;
    }

    for (size_t cl = 0; cl < res->class_num; ++cl) {
        free_cnbp(res->children[cl]);

        if (!(res->children[cl] = cnbp_from_file_with_params(
                file,
                class_num,
                categories,
                cat_num,
                nb_alpha,
                use_probs))) {
            printf("Failed to read children cnbp from file for bcc.\n");
            free_bcc(res);
            return NULL;
        }
    }

    return res;
}

bcc_t * bcc_from_file_with_name(
        const char * filename,
        uint8_t use_probs)
{
    FILE * file;
    bcc_t * res;

    if (!(file = fopen(filename, "r"))) {
        printf("Failed to open file %s for reading bcc model.\n", filename);
        return NULL;
    }

    if (!(res = bcc_from_file(
            file,
            use_probs)))
    {
        printf("Failed to read bcc from file: %s.\n", filename);
        fclose(file);
        return NULL;
    }

    fclose(file);
    return res;
}

void bcc_to_file(
        const bcc_t * bcc,
        FILE * file)
{
    fprintf(file, "%zu\n", bcc->cat_num);

    for (size_t cat = 0; cat < bcc->cat_num; ++cat) {
        fprintf(file, "%hhu\n", bcc->categories[cat]);
    }

    fprintf(file, "%hhu\n", bcc->class_num);
    fprintf(file, "%lf\n", bcc->nb_alpha);

    cnbp_to_file(bcc->root, file);

    for (size_t cl = 0; cl < bcc->class_num; ++cl) {
        cnbp_to_file(bcc->children[cl], file);
    }

    return;
}

void bcc_to_file_with_name(
        const bcc_t * bcc,
        const char * filename)
{
    FILE * file;

    if (!(file = fopen(filename, "w"))) {
        printf("Failed to open file %s for writing bcc model.\n", filename);
        return;
    }

    bcc_to_file(
            bcc,
            file);
    fclose(file);
    return;
}

static void bcc_train_single_with_scratch(
        bcc_t * bcc,
        const uint8_t * data,
        const double * label,
        double * root_scratch_space)
{
    double max = 0.0;
    uint8_t root_result = 0;

    cnbp_train_single(bcc->root, data, label);
    cnbp_predict_class(bcc->root, data, root_scratch_space);

    for (uint8_t cl = 0; cl < bcc->class_num; ++cl) {
        if (root_scratch_space[cl] > max) {
            root_result = cl;
            max = root_scratch_space[cl];
        }
    }

    cnbp_train_single(bcc->children[root_result], data, label);
}

