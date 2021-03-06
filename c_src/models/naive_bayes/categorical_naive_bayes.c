#include "categorical_naive_bayes.h"

#include <stdlib.h>
#include <string.h>
#include <math.h>

cnb_clas_t * create_cnb_with_alpha(uint8_t class_num, const uint8_t * cats, size_t csize, double alpha) {
    cnb_clas_t *res;
    uint32_t total_cat_values = 0;

    if (!(res = malloc(sizeof(cnb_clas_t)))) {
        printf("Failed to allocate memory for cnb classifier\n");
        return NULL;
    }

    memset(res, 0, sizeof(cnb_clas_t));

    if (!(res->categories = malloc(csize * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for cnb classifier categories\n");
        return NULL;
    }

    for (unsigned int cn = 0; cn < csize; ++cn) {
        res->categories[cn] = cats[cn];
    }

    if (!(res->class_totals = malloc(class_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for cnb classifier class totals\n");
        return NULL;
    }

    memset(res->class_totals, 0, class_num * sizeof(uint32_t));

    if (!(res->class_probs = malloc(class_num * sizeof(double)))) {
        printf("Failed to allocate memory for cnb classifier class probabilities\n");
        return NULL;
    }

    memset(res->class_probs, 0, class_num * sizeof(double));

    if (!(res->class_cat_idx = malloc(csize * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for cnb classifier category indices\n");
        return NULL;
    }

    memset(res->class_cat_idx, 0, csize * sizeof(uint32_t));

    for (unsigned int cn = 0; cn < csize; ++cn) {

        if (cn > 0) {
            res->class_cat_idx[cn] = res->class_cat_idx[cn - 1] + cats[cn - 1];
        }

        total_cat_values += cats[cn];
    }

    if (!(res->class_cat_totals = malloc(class_num * total_cat_values * sizeof(uint32_t)))) {
        printf("Failed to allocate memory cnb classifier class-wise category totals\n");
        return NULL;
    }

    memset(res->class_cat_totals, 0, class_num * total_cat_values * sizeof(uint32_t));

    if (!(res->class_cat_probs = malloc(class_num * total_cat_values * sizeof(double)))) {
        printf("Failed to allocate memory cnb classifier class-wise category probabilities\n");
        return NULL;
    }

    memset(res->class_cat_probs, 0, class_num * total_cat_values * sizeof(double));

    res->class_num = class_num;
    res->cat_num = csize;
    res->alpha = alpha;
    res->total_cat_vals = total_cat_values;

    recalculate_probabilities(res);

    return res;
}

void free_cnb(cnb_clas_t *cnb) {

    if (cnb->categories) {
        free(cnb->categories);
        cnb->categories = NULL;
    }

    if (cnb->class_totals) {
        free(cnb->class_totals);
        cnb->class_totals = NULL;
    }

    if (cnb->class_probs) {
        free(cnb->class_probs);
        cnb->class_probs = NULL;
    }

    if (cnb->class_cat_idx) {
        free(cnb->class_cat_idx);
        cnb->class_cat_idx = NULL;
    }

    if (cnb->class_cat_totals) {
        free(cnb->class_cat_totals);
        cnb->class_cat_totals = NULL;
    }

    if (cnb->class_cat_probs) {
        free(cnb->class_cat_probs);
        cnb->class_cat_probs = NULL;
    }

    free(cnb);
}

cnb_clas_t * create_cnb(uint8_t class_num, const uint8_t * cats, size_t csize) {
    return create_cnb_with_alpha(class_num, cats, csize, 1);
}

void train_batch(cnb_clas_t *cnb, const uint8_t * const * data, const uint8_t * labels, size_t dsize) {

    for (unsigned int dn = 0; dn < dsize; ++dn) {
        cnb->class_totals[labels[dn]] += 1;

        for (int cat = 0; cat < cnb->cat_num; ++cat) {
            cnb->class_cat_totals[labels[dn] * cnb->total_cat_vals + cnb->class_cat_idx[cat] + data[dn][cat]] += 1;
        }
    }

    recalculate_probabilities(cnb);

    printf("Categorical Naive Bayes trained on %zu data points\n", dsize);

    return;
}

void train_batch_on_selected(
        cnb_clas_t *cnb,
        const uint8_t * const * data,
        const uint8_t * labels,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected)
{
    unsigned int dn;

    if (dsize < num_selected) {
        printf("The data size (%zu) is lesser than the number of selected data points (%zu), so there must be an error\n", dsize, num_selected);
        return;
    }

    for (unsigned int sel_idx = 0; sel_idx < num_selected; ++sel_idx) {
        dn = selected_indices[sel_idx];
        cnb->class_totals[labels[dn]] += 1;

        for (int cat = 0; cat < cnb->cat_num; ++cat) {
            cnb->class_cat_totals[labels[dn] * cnb->total_cat_vals + cnb->class_cat_idx[cat] + data[dn][cat]] += 1;
        }

    }

    recalculate_probabilities(cnb);

    printf("Categorical Naive Bayes trained on %zu data points\n", num_selected);

    return;
}

void recalculate_probabilities(cnb_clas_t *cnb) {
    uint32_t cur_total = 0;

    for (int cl = 0; cl < cnb->class_num; ++cl) {
        cur_total += cnb->class_totals[cl];
    }

    for (int cl = 0; cl < cnb->class_num; ++cl) {

        cnb->class_probs[cl] = log((double) cnb->class_totals[cl] + cnb->alpha) - log((double) cur_total + cnb->class_num * cnb->alpha);

        for (int cat = 0; cat < cnb->cat_num; ++cat) {
            cur_total = 0;
            for (int val = 0; val < cnb->categories[cat]; ++val) {
                cur_total += cnb->class_cat_totals[cl * cnb->total_cat_vals + cnb->class_cat_idx[cat] + val];
            }

            for (int val = 0; val < cnb->categories[cat]; ++val) {
                cnb->class_cat_probs[cl * cnb->total_cat_vals + cnb->class_cat_idx[cat] + val] = log((double) (cnb->alpha +
                    cnb->class_cat_totals[cl * cnb->total_cat_vals + cnb->class_cat_idx[cat] + val])) -
                    log((double) (cur_total + cnb->categories[cat] * cnb->alpha));

            }
        }
    }

    return;
}

double * predict_class(const cnb_clas_t * cnb, const uint8_t * data) {
    double * res;

    if (!(res = malloc(cnb->class_num * sizeof(double)))) {
        printf("Failed to allocate memory for classifier prediction\n");
        return NULL;
    }


    for (int cl = 0; cl < cnb->class_num; ++cl) {
        res[cl] = (double) cnb->class_probs[cl];

        for (int cat = 0; cat < cnb->cat_num; ++cat) {
            res[cl] += (double) cnb->class_cat_probs[cl * cnb->total_cat_vals + cnb->class_cat_idx[cat] + data[cat]];
        }

    }

    return res;
}

cnb_clas_t * cnb_model_from_vals(const uint32_t * values, size_t num_values) {
    cnb_clas_t * res;
    uint8_t class_num;
    uint8_t cat_num;
    uint32_t alpha_num;
    uint32_t alpha_denom;
    uint8_t * categories;
    size_t at = 0;

    if (num_values < 4) {
        printf("Insufficient values given for a Categorical Naive Bayes model (less than %d, only %zu)\n", 4, num_values);
        return NULL;
    }

    class_num = (uint8_t) values[at++];
    alpha_num = values[at++];
    alpha_denom = values[at++];
    cat_num = (uint8_t) values[at++];

    printf("Loading Categorical Naive Bayes Classifier with %hhu classes and %hhu categories\n", class_num, cat_num);
    
    if (num_values < at + cat_num) {
        printf("Insufficient values given for a Categorical Naive Bayes model (less than %lu, only %zu)\n", at + cat_num, num_values);
        return NULL;
    }

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for Categorical Naive Bayes classifier categories\n");
        return NULL;
    }

    for (int cat = 0; cat < cat_num; ++cat) {
        categories[cat] = (uint8_t)values[at++];
    }

    if (!(res = create_cnb_with_alpha(class_num, categories, (size_t) cat_num, ((double)alpha_num) / alpha_denom))) {
        printf("Failed to create Categorical Naive Bayes classifier\n");
        free(categories);
        return NULL;
    }

    free(categories);

    if (num_values < at + class_num) {
        printf("Insufficient values given for a Categorical Naive Bayes model (less than %lu, only %zu)\n", at + class_num, num_values);
        free_cnb(res);
        return NULL;
    }

    for (int cl = 0; cl < class_num; ++cl) {
        res->class_totals[cl] = values[at++];
    }

    if (num_values < at + class_num * res->total_cat_vals) {
        printf("Insufficient values given for a Categorical Naive Bayes model (less than %lu, only %zu)\n", at + class_num * res->total_cat_vals, num_values);
        free_cnb(res);
        return NULL;
    }

    for (int cl = 0; cl < class_num; ++cl) {
        for (int cat = 0; cat < cat_num; ++cat) {
            for (int val = 0; val < res->categories[cat]; ++val) {
                res->class_cat_totals[cl * res->total_cat_vals + res->class_cat_idx[cat] + val] = values[at++];
            }
        }
    }

    recalculate_probabilities(res);

    return res;
}

uint32_t * cnb_model_to_vals(const cnb_clas_t * cnb) {
    size_t at = 0;
    uint32_t * res;
    size_t val_num;

    val_num = 4 + cnb->cat_num + cnb->class_num + cnb->total_cat_vals * cnb->class_num;

    if (!(res = malloc(val_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory to serialize Categorical Naive Bayes model\n");
        return NULL;
    }

    res[at++] = (uint32_t) cnb->class_num;

    if (cnb->alpha >= 1000) {
        res[at++] = (uint32_t)cnb->alpha;
        res[at++] = 1;
    }
    else if (cnb->alpha > 1) {
        res[at++] = (uint32_t)(cnb->alpha * 1000);
        res[at++] = 1000;
    }
    else {
        res[at++] = (uint32_t)(cnb->alpha * 1000000);
        res[at++] = 1000000;
    }

    res[at++] = (uint32_t) cnb->cat_num;

    for (int cat = 0; cat < cnb->cat_num; ++cat) {
        res[at++] = (uint32_t) cnb->categories[cat];
    }

    for (int cl = 0; cl < cnb->class_num; ++cl) {
        res[at++] = cnb->class_totals[cl];
    }

    for (int cl = 0; cl < cnb->class_num; ++cl) {
        for (int cat = 0; cat < cnb->cat_num; ++cat) {
            for (int val = 0; val < cnb->categories[cat]; ++val) {
                res[at++] = cnb->class_cat_totals[cl * cnb->total_cat_vals + cnb->class_cat_idx[cat] + val];
            }
        }
    }

    return res;
}

cnb_clas_t * cnb_from_file(
        FILE * file)
{

    cnb_clas_t * res;
    uint8_t class_num;
    uint8_t cat_num;
    uint32_t alpha_num;
    uint32_t alpha_denom;
    uint8_t * categories;

    fscanf(file, "%hhu\n", &class_num);
    fscanf(file, "%u\n", &alpha_num);
    fscanf(file, "%u\n", &alpha_denom);

    fscanf(file, "%hhu\n", &cat_num);

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for categories array for Naive Bayes Classifier.\n");
        return NULL;
    }

    memset(categories, 0, cat_num * sizeof(uint8_t));


    for (uint8_t cat = 0; cat < cat_num; ++cat) {
        fscanf(file, "%hhu\n", &categories[cat]);
    }

    if (!(res = create_cnb_with_alpha(class_num, categories, (size_t) cat_num, ((double)alpha_num) / alpha_denom))) {
        printf("Failed to create Categorical Naive Bayes classifier\n");
        free(categories);
        return NULL;
    }

    free(categories);

    for (uint8_t class = 0; class < res->class_num; ++class) {
        fscanf(file, "%u\n", &(res->class_totals[class]));
    }

    for (int cl = 0; cl < class_num; ++cl) {
        for (int cat = 0; cat < cat_num; ++cat) {
            for (int val = 0; val < res->categories[cat]; ++val) {
                fscanf(
                        file,
                        "%u\n",
                        &(res->class_cat_totals[cl * res->total_cat_vals + res->class_cat_idx[cat] + val]));
            }
        }
    }

    recalculate_probabilities(res);

    return res;
}

void cnb_to_file(
        const cnb_clas_t * cnb,
        FILE * file)
{
    fprintf(file, "%hhu\n", cnb->class_num);

    if (cnb->alpha >= 1000) {
        fprintf(file, "%u\n", (uint32_t)cnb->alpha);
        fprintf(file, "%u\n", 1);
    }
    else if (cnb->alpha > 1) {
        fprintf(file, "%u\n", (uint32_t)(cnb->alpha * 1000));
        fprintf(file, "%u\n", 1000);
    }
    else {
        fprintf(file, "%u\n", (uint32_t)(cnb->alpha * 1000000));
        fprintf(file, "%u\n", 1000000);
    }

    fprintf(file, "%hhu\n", cnb->cat_num);

    for (int cat = 0; cat < cnb->cat_num; ++cat) {
        fprintf(file, "%hhu\n", cnb->categories[cat]);
    }

    for (int cl = 0; cl < cnb->class_num; ++cl) {
        fprintf(file, "%u\n", cnb->class_totals[cl]);
    }

    for (int cl = 0; cl < cnb->class_num; ++cl) {
        for (int cat = 0; cat < cnb->cat_num; ++cat) {
            for (int val = 0; val < cnb->categories[cat]; ++val) {
                fprintf(
                        file,
                        "%u\n",
                        cnb->class_cat_totals[cl * cnb->total_cat_vals + cnb->class_cat_idx[cat] + val]);
            }
        }
    }

    fflush(file);

    return;
}

