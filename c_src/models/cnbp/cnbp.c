#include "cnbp.h"

#include <stdlib.h>
#include <string.h>
#include <math.h>

static void recalculate_probabilities(cnbp_t *cnbp);

cnbp_t * create_cnbp_with_alpha(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        double alpha,
        uint8_t use_probs)
{
    cnbp_t *res;
    uint32_t total_cat_values = 0;

    if (!(res = malloc(sizeof(cnbp_t)))) {
        printf("Failed to allocate memory for cnbp classifier\n");
        return NULL;
    }

    memset(res, 0, sizeof(cnbp_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for cnbp classifier categories\n");
        free_cnbp(res);
        return NULL;
    }

    for (uint32_t cn = 0; cn < cat_num; ++cn) {
        res->categories[cn] = cats[cn];
    }

    if (!(res->class_totals = malloc(class_num * sizeof(double)))) {
        printf("Failed to allocate memory for cnbp classifier class totals\n");
        free_cnbp(res);
        return NULL;
    }

    memset(res->class_totals, 0, class_num * sizeof(double));

    if (use_probs) {

        if (!(res->class_probs = malloc(class_num * sizeof(double)))) {
            printf("Failed to allocate memory for cnbp classifier class probabilities\n");
            free_cnbp(res);
            return NULL;
        }

        memset(res->class_probs, 0, class_num * sizeof(double));

    }

    if (!(res->class_cat_idx = malloc(cat_num * sizeof(uint32_t)))) {
        printf("Failed to allocate memory for cnbp classifier category indices\n");
        free_cnbp(res);
        return NULL;
    }

    memset(res->class_cat_idx, 0, cat_num * sizeof(uint32_t));

    for (unsigned int cn = 0; cn < cat_num; ++cn) {

        if (cn > 0) {
            res->class_cat_idx[cn] = res->class_cat_idx[cn - 1] + cats[cn - 1];
        }

        total_cat_values += cats[cn];
    }

    if (!(res->class_cat_totals = malloc((class_num * total_cat_values) * sizeof(double)))) {
        printf("Failed to allocate memory cnbp classifier class-wise category totals\n");
        return NULL;
    }

    memset(res->class_cat_totals, 0, class_num * total_cat_values * sizeof(double));

    if (use_probs) {

        if (!(res->class_cat_probs = malloc((class_num * total_cat_values) * sizeof(double)))) {
            printf("Failed to allocate memory cnbp classifier class-wise category probabilities\n");
            return NULL;
        }

        memset(res->class_cat_probs, 0, (class_num * total_cat_values) * sizeof(double));
    }

    res->class_num = class_num;
    res->cat_num = (uint32_t) cat_num;
    res->alpha = alpha;
    res->total_cat_vals = total_cat_values;
    res->is_fresh = 0;
    res->use_probs = use_probs;

    if (use_probs) {
        recalculate_probabilities(res);
    }

    return res;
}

cnbp_t * create_cnbp(
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        uint8_t use_probs)
{
    return create_cnbp_with_alpha(
            class_num,
            cats,
            cat_num,
            1.0,
            use_probs);
}

void cnbp_train_single(
        cnbp_t * cnbp,
        const uint8_t * data,
        const double * label)
{
    double labels_check = 0;

    for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
        labels_check += label[cl];
    }

    if (fabs(labels_check - 1.0) > .000001) {
        printf("Attempt to train cnbp on datapoint failed, because provided labels sum was not within the threshold, sum: %lf\n", labels_check);
        return;
    }

    for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
        cnbp->class_totals[cl] += label[cl];

        for (uint32_t cat = 0; cat < cnbp->cat_num; ++cat) {
            cnbp->class_cat_totals[cl * cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + data[cat]] += label[cl];
        }
    }

    cnbp->is_fresh = 0;
    return;
}

void cnbp_train_batch(
        cnbp_t * cnbp,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize)
{
    for (size_t d = 0; d < dsize; ++d) {

        cnbp_train_single(
                cnbp,
                datas[d],
                labels[d]);

    }

    return;
}

void cnbp_train_batch_on_selected(
        cnbp_t * cnbp,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize,
        const size_t * selected_indices,
        size_t num_selected)
{
    if (num_selected >= dsize) {
        printf("Number of selected datapoints for training exceeds the total number of datapoints.\n");
        return;
    }

    for (size_t s = 0; s < num_selected; ++s) {
        cnbp_train_single(
                cnbp,
                datas[selected_indices[s]],
                labels[selected_indices[s]]);
    }

    return;
}

void cnbp_predict_class(
        const cnbp_t * cnbp,
        const uint8_t * data,
        double * labels)
{
    double total = 0;
    double class_total = 0;

    if (!cnbp->is_fresh && cnbp->use_probs) {
        recalculate_probabilities((cnbp_t *) cnbp);
    }

/** Alternatively we can avoid this and simply do the multiplication on doubles **/
    if (cnbp->use_probs) {

        for (uint8_t cl = 0; cl < cnbp-> class_num; ++cl) {
            labels[cl] = (double) cnbp->class_probs[cl];

            for (uint32_t cat = 0; cat < cnbp->cat_num; ++cat) {
                labels[cl] *= (double) cnbp->class_cat_probs[
                cl *cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + data[cat]];
            }

            total += labels[cl];
        }

    }
    else {

        for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
            class_total += cnbp->class_totals[cl];
        }

        for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {

            labels[cl] = (cnbp->class_totals[cl] + cnbp->alpha)/(class_total + (cnbp->class_num * cnbp->alpha));

            for (uint32_t cat = 0; cat < cnbp->cat_num; ++cat) {

                labels[cl] *= ((cnbp->class_cat_totals[cl *cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + data[cat]] + cnbp->alpha) /
                    (cnbp->class_totals[cl] + (cnbp->categories[cat] * cnbp->alpha)));
            }

            total += labels[cl];
        }

    }


/**
 * This implementation assumes probabilties are stored as logs.
 */
//    if (use_probs) {
//
//        for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
//            labels[cl] = (double) cnbp->class_probs[cl];
//
//            for (uint32_t cat = 0; cat < cnbp->cat_num; ++cat) {
//                labels[cl] += (double) cnbp->class_cat_probs[cl * cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + data[cat]];
//            }
//
//            labels[cl] = pow(10, labels[cl]);
//            total += labels[cl];
//        }
//    }
//    else {
//
//        for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
//            class_total += cnbp->class_totals[cl];
//        }
//
//        for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
//            labels[cl] = log10(cnbp->class_totals[cl] + cnbp->alpha) - log10(class_total + cnbp->class_num * cnbp->alpha);
//
//            for (uint32_t cat = 0; cat < cnbp->cat_num; ++cat) {
//                labels[cl] += (log10(cnbp->class_cat_totals[cl * cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + data[cat]] + cnbp->alpha) -
//                        log10(cnbp->class_totals[cl] + cnbp->categories[cat] * cnbp->alpha));
//            }
//
//            labels[cl] = pow(10, labels[cl]);
//            total += labels[cl];
//        }
//    }

    for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
        labels[cl] /= total;
    }

    return;
}

cnbp_t * cnbp_from_file_with_params(
        FILE * file,
        uint8_t class_num,
        const uint8_t * cats,
        size_t cat_num,
        double alpha,
        uint8_t use_probs)
{
    cnbp_t * res;

    if (!(res = create_cnbp_with_alpha(class_num, cats, cat_num, alpha, use_probs))) {
        printf("Failed to create cnbp classifier\n");
        return NULL;
    }

    for (uint8_t class = 0; class < res->class_num; ++class) {
        fscanf(file, "%lf\n", &(res->class_totals[class]));
    }

    for (uint8_t cl = 0; cl < class_num; ++cl) {
        for (uint32_t cat = 0; cat < cat_num; ++cat) {
            for (uint8_t val = 0; val < res->categories[cat]; ++val) {
                fscanf(
                        file,
                        "%lf\n",
                        &(res->class_cat_totals[cl * res->total_cat_vals + res->class_cat_idx[cat] + val]));
            }
        }
    }

    if (use_probs) {
        recalculate_probabilities(res);
    }

    return res;
}

cnbp_t * cnbp_from_file(
        FILE * file,
        uint8_t use_probs)
{
    cnbp_t * res;
    uint8_t class_num;
    uint32_t cat_num;
    double alpha;
    uint8_t * categories;

    fscanf(file, "%hhu\n", &class_num);
    fscanf(file, "%lf\n", &alpha);
    fscanf(file, "%u\n", &cat_num);

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for categories array for cnbp Classifier.\n");
        return NULL;
    }

    memset(categories, 0, cat_num * sizeof(uint8_t));


    for (uint32_t cat = 0; cat < cat_num; ++cat) {
        fscanf(file, "%hhu\n", &categories[cat]);
    }

    if (!(res = cnbp_from_file_with_params(
            file,
            class_num,
            categories,
            cat_num,
            alpha,
            use_probs))) {
        printf("Failed read cnbp values from file.\n");
        free(categories);
        return NULL;
    }

    free(categories);
    return res;
}

cnbp_t * cnbp_from_file_with_name(
        const char * filename,
        uint8_t use_probs)
{
    FILE * file;
    cnbp_t * res;

    if (!(file = fopen(filename, "r"))) {
        printf("Failed to open file %s for reading cnbp model.\n", filename);
        return NULL;
    }

    if (!(res = cnbp_from_file(
            file,
            use_probs)))
    {
        printf("Failed to read cnbp from file: %s.\n", filename);
        fclose(file);
        return NULL;
    }

    fclose(file);
    return res;
}

void cnbp_to_file_no_params(
        const cnbp_t * cnbp,
        FILE * file)
{
    for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
        fprintf(file, "%lf\n", cnbp->class_totals[cl]);
    }


    for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {

        for (uint32_t cat = 0; cat < cnbp->cat_num; ++cat) {
            for (uint8_t val = 0; val < cnbp->categories[cat]; ++val) {
                fprintf(file, "%lf\n", cnbp->class_cat_totals[cl * cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + val]);
            }
        }
    }

    return;
}

void cnbp_to_file(
        const cnbp_t * cnbp,
        FILE * file)
{
    fprintf(file, "%hhu\n", cnbp->class_num);
    fprintf(file, "%lf\n", cnbp->alpha);
    fprintf(file, "%u\n", cnbp->cat_num);

    for (uint8_t cat = 0; cat < cnbp->cat_num; ++cat) {
        fprintf(file, "%hhu\n", cnbp->categories[cat]);
    }

    cnbp_to_file_no_params(
            cnbp,
            file);

    return;
}


void cnbp_to_file_with_name(
        const cnbp_t * cnbp,
        const char * filename)
{
    FILE * file;

    if (!(file = fopen(filename, "w"))) {
        printf("Failed to open file %s for writing cnbp model.\n", filename);
        return;
    }

    cnbp_to_file(
            cnbp,
            file);
    return;
}

void cnbp_forget(
        cnbp_t * cnbp,
        double factor)
{
    for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
        cnbp->class_totals[cl] /= factor;
    }

    for (size_t catval = 0; catval < (cnbp->class_num * cnbp->total_cat_vals); ++catval) {
        cnbp->class_cat_totals[catval] /= factor;
    }

    if (cnbp->use_probs) {
        recalculate_probabilities(cnbp);
    }

    return;
}

cnbp_t * copy_cnbp(const cnbp_t * cnbp) {
    cnbp_t * res;

    if (!(res = create_cnbp_with_alpha(
            cnbp->class_num,
            cnbp->categories,
            cnbp->cat_num,
            cnbp->alpha,
            cnbp->use_probs)))
    {
        printf("Failed to create new cnbp classifier to make a copy\n");
        return NULL;
    }

    for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {
        res->class_totals[cl] = cnbp->class_totals[cl];
    }

    for (uint32_t catval = 0; catval < (cnbp->class_num * cnbp->total_cat_vals); ++catval) {
        res->class_cat_totals[catval] = cnbp->class_cat_totals[catval];
    }

    if (cnbp->use_probs) {
        recalculate_probabilities(res);
    }

    return res;
}

void free_cnbp(cnbp_t * cnbp) {

    if (!cnbp) {
        return;
    }

    if (cnbp->categories) {
        free(cnbp->categories);
        cnbp->categories = NULL;
    }

    if (cnbp->class_totals) {
        free(cnbp->class_totals);
        cnbp->class_totals = NULL;
    }

    if (cnbp->class_probs) {
        free(cnbp->class_probs);
        cnbp->class_probs = NULL;
    }

    if (cnbp->class_cat_idx) {
        free(cnbp->class_cat_idx);
        cnbp->class_cat_idx = NULL;
    }

    if (cnbp->class_cat_totals) {
        free(cnbp->class_cat_totals);
        cnbp->class_cat_totals = NULL;
    }

    if (cnbp->class_cat_probs) {
        free(cnbp->class_cat_probs);
        cnbp->class_cat_probs = NULL;
    }

    free(cnbp);
    return;
}

void recalculate_probabilities(cnbp_t *cnbp) {
    double cur_total = 0;

    for (int cl = 0; cl < cnbp->class_num; ++cl) {
        cur_total += cnbp->class_totals[cl];
    }

/** The following implementation is for the solution using logs to store probabilities **/
//    for (int cl = 0; cl < cnbp->class_num; ++cl) {
//
//        cnbp->class_probs[cl] = log10(cnbp->class_totals[cl] + cnbp->alpha) - log10(cur_total + cnbp->class_num * cnbp->alpha);
//
//        for (uint32_t cat = 0; cat < cnbp->cat_num; ++cat) {
//
//            for (int val = 0; val < cnbp->categories[cat]; ++val) {
//                cnbp->class_cat_probs[cl * cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + val] = log10(cnbp->alpha +
//                    cnbp->class_cat_totals[cl * cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + val]) -
//                    log10(cnbp->class_totals[cl] + cnbp->categories[cat] * cnbp->alpha);
//
//            }
//
//        }
//    }

/** Alternative solution which stores exact probabilities rather than their logs **/
    for (uint8_t cl = 0; cl < cnbp->class_num; ++cl) {

        cnbp->class_probs[cl] = (cnbp->class_totals[cl] + cnbp->alpha) / (cur_total + cnbp->class_num * cnbp->alpha);

        for (uint8_t cat = 0; cat < cnbp->cat_num; ++cat) {

            for (uint8_t val = 0; val < cnbp->categories[cat]; ++val) {
                cnbp->class_cat_probs[cl * cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + val] = (cnbp->alpha +
                      cnbp->class_cat_totals[cl * cnbp->total_cat_vals + cnbp->class_cat_idx[cat] + val]) /
                      (cnbp->class_totals[cl] + cnbp->categories[cat] * cnbp->alpha);

            }
        }
    }

    cnbp->is_fresh = 1;
    return;
}
