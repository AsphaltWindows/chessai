#include "bdt2.h"

#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

static bdt2_node_t * create_bdt2_node_shell(
        const bdt2_t * bdt2);

static bdt2_node_t * create_bdt2_node_leaf(
        const bdt2_t * bdt2);

static void node_train_single(
        bdt2_t * bdt2,
        bdt2_node_t * node,
        const uint8_t * data,
        const double * label,
        double error);

static void node_predict_class(
        const bdt2_t * bdt2,
        bdt2_node_t * node,
        const uint8_t * data,
        double * labels);

static void free_bdt2_node(
        bdt2_node_t * node);

static bdt2_node_t * bdt2_node_from_file(
        const bdt2_t * bdt2,
        FILE * file);

static void bdt2_node_to_file(
        const bdt2_node_t * node,
        FILE * file);

static double bdt2_compute_error(
        bdt2_t * bdt2,
        const double * expected,
        const double * actual);


bdt2_t * create_bdt2(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        uint32_t split_threshold,
        uint32_t split_limit,
        double nb_alpha,
        uint8_t use_probs)
{

    bdt2_t * res;

    if (!(res = malloc(sizeof(bdt2_t)))) {
        printf("Failed to allocate memory for new bdt2 classifier\n");
        return NULL;
    }

    memset(res, 0, sizeof(bdt2_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for categories array for bdt2 classifier.\n");
        free_bdt2(res);
        return NULL;
    }

    memset(res->categories, 0, cat_num * sizeof(uint8_t));

    for (size_t cat = 0; cat < cat_num; ++cat) {
        res->categories[cat] = categories[cat];
    }

    if (!(res->class_scratch_space = malloc(class_num * sizeof(double)))) {
        printf("Failed to allocate memory for bdt2 class scratch space.\n");
        free_bdt2(res);
        return NULL;
    }

    memset(res->class_scratch_space, 0, class_num * sizeof(double));

    if (!(res->branch_scratch_space = malloc(branch_factor * sizeof(double)))) {
        printf("Failed to allocate memory for bdt2 branch scratch space.\n");
        free_bdt2(res);
        return NULL;
    }

    memset(res->branch_scratch_space, 0, branch_factor * sizeof(double));

    res->cat_num = (uint32_t) cat_num;
    res->class_num = class_num;
    res->branch_factor = branch_factor;
    res->split_threshold = split_threshold;
    res->split_limit = split_limit;
    res->split_number = 0;
    res->forget_factor = forget_factor;
    res->nb_alpha = nb_alpha;
    res->use_probs = use_probs;


    if (!(res->nodes = malloc(1 * sizeof(bdt2_node_t *)))) {
        printf("Failed to allocate memory for bdt2 nodes array.\n");
        free_bdt2(res);
        return NULL;
    }

    memset(res->nodes, 0, 1 * sizeof(bdt2_node_t *));

    if (!(res->nodes[0] = create_bdt2_node_leaf(res))) {
        printf("Failed to create initial bdt2 classifier node.\n");
        free_bdt2(res);
        return NULL;
    }

    res->nodes[0]->node_id = 0;
    res->nodes_num = 1;

    return res;
}

void bdt2_train_single(
        bdt2_t * bdt2,
        const uint8_t * data,
        const double * label)
{
    double child_error;

    node_predict_class(
            bdt2,
            bdt2->nodes[0],
            data,
            bdt2->class_scratch_space);

    child_error = bdt2_compute_error(
            bdt2,
            bdt2->class_scratch_space,
            label);

    node_train_single(
            bdt2,
            bdt2->nodes[0],
            data,
            label,
            child_error);

    return;
}

void bdt2_train_batch(
        bdt2_t * bdt2,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize)
{
    uint64_t start_time;
    uint64_t end_time;

//    printf("BDT training batch on %zu data points.\n", dsize);
    start_time = time(NULL);

    for (size_t d = 0; d < dsize; ++d) {
        bdt2_train_single(
                bdt2,
                datas[d],
                labels[d]);
    }

    end_time = time(NULL);

//    printf("Time spent training: %lu\n", end_time - start_time);

    return;
}

void bdt2_predict_class(
        const bdt2_t * bdt2,
        const uint8_t * data,
        double * labels)
{
    return node_predict_class(
            bdt2,
            bdt2->nodes[0],
            data,
            labels);
}

void free_bdt2(
        bdt2_t * bdt2)
{
    if (!bdt2) {
        return;
    }

    if (bdt2->categories) {
        free(bdt2->categories);
        bdt2->categories = NULL;
    }

    if (bdt2->class_scratch_space) {
        free(bdt2->class_scratch_space);
        bdt2->class_scratch_space = NULL;
    }

    if (bdt2->branch_scratch_space) {
        free(bdt2->branch_scratch_space);
        bdt2->branch_scratch_space = NULL;
    }

    if (bdt2->nodes) {
        for (size_t n = 0; n < bdt2->nodes_num; ++n) {
            free_bdt2_node(bdt2->nodes[n]);
            bdt2->nodes[n] = NULL;
        }
        free(bdt2->nodes);
        bdt2->nodes = NULL;
    }

    free(bdt2);
    return;
}

bdt2_t * bdt2_from_file(
        FILE * file,
        uint8_t use_probs)
{
    bdt2_t * res;
    uint8_t * categories;
    uint32_t cat_num;
    uint8_t class_num;
    uint8_t branch_factor;
    uint32_t split_threshold;
    uint32_t split_limit;
    uint32_t split_number;
    size_t nodes_num;
    double forget_factor;
    double nb_alpha;

    fscanf(file, "%u\n", &cat_num);

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for categories of bdt2 classifier being read from file.\n");
        return NULL;
    }

    for (uint32_t cat = 0; cat < cat_num; ++cat) {
        fscanf(file, "%hhu\n", &(categories[cat]));
    }

    fscanf(file, "%hhu\n", &class_num);
    fscanf(file, "%hhu\n", &branch_factor);
    fscanf(file, "%u\n", &split_threshold);
    fscanf(file, "%u\n", &split_limit);
    fscanf(file, "%u\n", &split_number);
    fscanf(file, "%zu\n", &nodes_num);
    fscanf(file, "%lf\n", &forget_factor);
    fscanf(file, "%lf\n", &nb_alpha);

    if (!(res = create_bdt2(
            categories,
            cat_num,
            class_num,
            branch_factor,
            split_threshold,
            split_limit,
            forget_factor,
            nb_alpha,
            use_probs)))
    {
        printf("Failed to allocate memory for bdt2 classifier being read from file.\n");
        free(categories);
        return NULL;
    }

    free_bdt2_node(res->nodes[0]);
    free(res->nodes);

    res->nodes_num = nodes_num;
    res->split_number = split_number;

    if (!(res->nodes = malloc(nodes_num * sizeof(bdt2_node_t *)))) {
        printf("Failed to allocate memory for bdt2 node array while reading bdt2 classifier from file.\n");
        free_bdt2(res);
        return NULL;
    }

    for (size_t n = 0; n < nodes_num; ++n) {
        if (!(res->nodes[n] = bdt2_node_from_file(
                res,
                file)))
        {
            printf("Failed to read bdt2 node from file.\n");
            free_bdt2(res);
            return NULL;
        }
    }

    return res;
}


bdt2_t * bdt2_from_file_with_name(
        const char * filename,
        uint8_t use_probs)
{
    FILE * file;
    bdt2_t * res;

    if (!(file = fopen(filename, "r"))) {
        printf("Failed to open file %s for reading bdt2 model.\n", filename);
        return NULL;
    }

    if (!(res = bdt2_from_file(
            file,
            use_probs)))
    {
        printf("Failed to read bdt2 from file: %s.\n", filename);
        fclose(file);
        return NULL;
    }

    fclose(file);
    return res;
}

void bdt2_to_file(
        const bdt2_t * bdt2,
        FILE * file)
{
    fprintf(file, "%u\n", bdt2->cat_num);

    for (uint32_t cat = 0; cat < bdt2->cat_num; ++cat) {
        fprintf(file, "%hhu\n", bdt2->categories[cat]);
    }

    fprintf(file, "%hhu\n", bdt2->class_num);
    fprintf(file, "%hhu\n", bdt2->branch_factor);
    fprintf(file, "%u\n", bdt2->split_threshold);
    fprintf(file, "%u\n", bdt2->split_limit);
    fprintf(file, "%u\n", bdt2->split_number);
    fprintf(file, "%zu\n", bdt2->nodes_num);
    fprintf(file, "%lf\n", bdt2->forget_factor);
    fprintf(file, "%lf\n", bdt2->nb_alpha);

    for (size_t n = 0; n < bdt2->nodes_num; ++n) {
        bdt2_node_to_file(
                bdt2->nodes[n],
                file);
    }

    return;
}

void bdt2_to_file_with_name(
        const bdt2_t * bdt2,
        const char * filename)
{
    FILE * file;

    if (!(file = fopen(filename, "w"))) {
        printf("Failed to open file %s for writing bdt2 model.\n", filename);
        return;
    }

    bdt2_to_file(
            bdt2,
            file);
    return;
}

static bdt2_node_t * create_bdt2_node_shell(
        const bdt2_t * bdt2)
{
    bdt2_node_t * res;

    if (!(res = malloc(sizeof(bdt2_node_t)))) {
        printf("Failed to allocate memory for bdt2 node.\n");
        return NULL;
    }

    memset(res, 0, sizeof(bdt2_node_t));

    res->root_bdt2 = (bdt2_t *) bdt2;
    res->children = NULL;
    res->num_trained_total = 0;
    res->num_trained_last = 0;
    res->error_total = 0.0;
    res->error_last = 0.0;

    return res;
}

static bdt2_node_t * create_bdt2_node_leaf(
        const bdt2_t * bdt2)
{
    bdt2_node_t * res;

    if (!(res = create_bdt2_node_shell(bdt2))) {
        printf("Failed to create shell bdt2 node.\n");
        return NULL;
    }

    res->type = LEAF;
    if (!(res->classifier = create_cnbp_with_alpha(
            bdt2->class_num,
            bdt2->categories,
            bdt2->cat_num,
            bdt2->nb_alpha,
            bdt2->use_probs)))
    {
        printf("Failed to create cnbp for bdt2 node.\n");
        free_bdt2_node(res);
        return NULL;
    }

    return res;
}

static void node_train_single(
        bdt2_t * bdt2,
        bdt2_node_t * node,
        const uint8_t * data,
        const double * label,
        double error)
{
    double child_error;
    double min_error;
    uint8_t min_error_index;
    double * expected;

    if (node->type == LEAF) {

        if (node->num_trained_last >= bdt2->split_threshold && bdt2->split_number < bdt2->split_limit) {

            if ((node->error_last / node->num_trained_last) > (node->error_total / node->num_trained_total)) {

                printf("Splitting Node with node_id: %u.\n", node->node_id);

                if (!(node->children = malloc(bdt2->branch_factor * sizeof(uint32_t)))) {
                    printf("Failed to allocate array for children indices.\n");
                    return;
                }

                if (!(bdt2->nodes = realloc(
                        bdt2->nodes,
                        (bdt2->nodes_num + bdt2->branch_factor) * sizeof(bdt2_node_t *)))) {
                    printf("Failed to reallocate the nodes array of bdt2.\n");
                    return;
                }

                for (uint8_t child = 0; child < bdt2->branch_factor; ++child) {
                    node->children[child] = bdt2->nodes_num + child;

                    if (!(bdt2->nodes[bdt2->nodes_num + child] = create_bdt2_node_shell(bdt2))) {
                        printf("Failed to create new bdt2 node shell for node split.\n");
                        return;
                    }

                    bdt2->nodes[bdt2->nodes_num + child]->node_id = bdt2->nodes_num + child;
                    bdt2->nodes[bdt2->nodes_num + child]->type = LEAF;

                    if (!(bdt2->nodes[bdt2->nodes_num + child]->classifier = copy_cnbp(node->classifier))) {
                        printf("Failed to copy cnbp of bdt2 node for node split.\n");
                        return;
                    }

                    cnbp_forget(
                            bdt2->nodes[bdt2->nodes_num + child]->classifier,
                            bdt2->forget_factor);
                }

                free_cnbp(node->classifier);

                if (!(node->classifier = create_cnbp_with_alpha(
                        bdt2->branch_factor,
                        bdt2->categories,
                        bdt2->cat_num,
                        bdt2->nb_alpha,
                        bdt2->use_probs))) {
                    printf("Failed to create new classifier for branch node.\n");
                    return;
                }

                node->type = BRANCH;
                node->num_trained_total = 0;
                node->error_total = 0;
                bdt2->nodes_num += bdt2->branch_factor;
                bdt2->split_number++;
            }

            node->num_trained_last = 0;
            node->error_last = 0.0;
            return node_train_single(
                    bdt2,
                    node,
                    data,
                    label,
                    error);
        }
        else {
            cnbp_train_single(
                    node->classifier,
                    data,
                    label);
            node->num_trained_last++;
            node->num_trained_total++;
            node->error_last += error;
            node->error_total += error;
            return;
        }
    }
    else if (node->type == BRANCH) {

        for (uint8_t child = 0; child < bdt2->branch_factor; ++child) {

            node_predict_class(
                    bdt2,
                    bdt2->nodes[node->children[child]],
                    data,
                    bdt2->class_scratch_space);
            expected = bdt2->class_scratch_space;
            child_error = 0.0;

            for (uint8_t cl = 0; cl < bdt2->class_num; ++cl) {
                child_error += fabs(expected[cl] - label[cl]);
            }

            child_error /= 2;

            if (!child || child_error < min_error) {
                min_error = child_error;
                min_error_index = child;
            }

        }

        for (uint8_t b = 0; b < bdt2->branch_factor; ++b) {
            bdt2->branch_scratch_space[b] = 0.0;
        }

        bdt2->branch_scratch_space[min_error_index] = 1.0;

        cnbp_train_single(
                node->classifier,
                data,
                bdt2->branch_scratch_space);

        node_train_single(
                bdt2,
                bdt2->nodes[node->children[min_error_index]],
                data,
                label,
                min_error);

        node->num_trained_last++;
        node->num_trained_total++;
        node->error_last += error;
        node->error_total += error;
        return;
    }

    return;
}

static void node_predict_class(
        const bdt2_t * bdt2,
        bdt2_node_t * node,
        const uint8_t * data,
        double * labels)
{
    double * branch_expected;
    double max;
    uint8_t max_index;

    if (node->type == LEAF) {

        cnbp_predict_class(
                node->classifier,
                data,
                labels);
    }
    else if (node->type == BRANCH) {

        cnbp_predict_class(
                node->classifier,
                data,
                bdt2->branch_scratch_space);
        branch_expected = bdt2->branch_scratch_space;

        for (uint8_t b = 0; b < bdt2->branch_factor; ++b) {

            if (!b || branch_expected[b] > max) {
                max = branch_expected[b];
                max_index = b;
            }

        }

        node_predict_class(
                bdt2,
                bdt2->nodes[node->children[max_index]],
                data,
                labels);
    }
    else {
        printf("Unrecognized bdt2 NODE TYPE: %u.\n", node->type);
    }

    return;
}

static void free_bdt2_node(
        bdt2_node_t * node)
{
    if (!node) {
        return;
    }

    if (node->children) {
        free(node->children);
        node->children = NULL;
    }

    if (node->classifier) {
        free_cnbp(node->classifier);
        node->classifier = NULL;
    }

    free(node);
    return;
}

static bdt2_node_t * bdt2_node_from_file(
        const bdt2_t * bdt2,
        FILE * file)
{
    bdt2_node_t * res;

    if (!(res = create_bdt2_node_shell(bdt2))) {
        printf("Failed to create bdt2 node shell.\n");
        return NULL;
    }

    fscanf(file, "%u\n", &(res->type));
    fscanf(file, "%u\n", &(res->node_id));
    fscanf(file, "%u\n", &(res->num_trained_total));
    fscanf(file, "%lf\n", &(res->error_total));
    fscanf(file, "%u\n", &(res->num_trained_last));
    fscanf(file, "%lf\n", &(res->error_last));

    if (res->type == BRANCH) {

        if (!(res->children = malloc(bdt2->branch_factor * sizeof(uint32_t)))) {
            printf("Failed to allocate children array for bdt2 node while reading from file.\n");
            free_bdt2_node(res);
            return NULL;
        }

        memset(res->children, 0, bdt2->branch_factor * sizeof(uint32_t));

        for (uint8_t child = 0; child < bdt2->branch_factor; ++child) {
            fscanf(file, "%u\n", &(res->children[child]));
        }
    }

    if (!(res->classifier = cnbp_from_file_with_params(
            file,
            res->type == LEAF ? bdt2->class_num : bdt2->branch_factor,
            bdt2->categories,
            bdt2->cat_num,
            bdt2->nb_alpha,
            bdt2->use_probs)))
    {
        printf("Failed to read cnbp from file for bdt2 node.\n");
        free_bdt2_node(res);
    }

    return res;
}

static void bdt2_node_to_file(
        const bdt2_node_t * node,
        FILE * file)
{
    fprintf(file, "%u\n", (uint32_t) node->type);
    fprintf(file, "%u\n", node->node_id);
    fprintf(file, "%u\n", node->num_trained_total);
    fprintf(file, "%lf\n", node->error_total);
    fprintf(file, "%u\n", node->num_trained_last);
    fprintf(file, "%lf\n", node->error_last);

    if (node->type == BRANCH) {

        for (uint8_t child = 0; child < node->root_bdt2->branch_factor; ++child) {
            fprintf(file, "%u\n", node->children[child]);
        }

    }

    cnbp_to_file_no_params(
            node->classifier,
            file);
    return;
}


static double bdt2_compute_error(
        bdt2_t * bdt2,
        const double * expected,
        const double * actual)
{
    double child_error = 0.0;

    for (uint8_t cl = 0; cl < bdt2->class_num; ++cl) {
        child_error += fabs(expected[cl] - actual[cl]);
    }

    return child_error /= 2;
}
