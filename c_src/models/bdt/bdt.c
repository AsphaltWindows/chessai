#include "bdt.h"

#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

static bdt_node_t * create_bdt_node_shell(
        const bdt_t * bdt);

static bdt_node_t * create_bdt_node_leaf(
        const bdt_t * bdt);

static void node_train_single(
        bdt_t * bdt,
        bdt_node_t * node,
        const uint8_t * data,
        const double * label);

static void node_predict_class(
        const bdt_t * bdt,
        bdt_node_t * node,
        const uint8_t * data,
        double * labels);

static void free_bdt_node(
        bdt_node_t * node);

static bdt_node_t * bdt_node_from_file(
        const bdt_t * bdt,
        FILE * file);

static void bdt_node_to_file(
        const bdt_node_t * node,
        FILE * file);

bdt_t * create_bdt(
        uint8_t * categories,
        size_t cat_num,
        uint8_t class_num,
        uint8_t branch_factor,
        uint32_t split_threshold,
        uint32_t split_limit,
        double forget_factor,
        double nb_alpha,
        uint8_t use_probs)
{

    bdt_t * res;

    if (!(res = malloc(sizeof(bdt_t)))) {
        printf("Failed to allocate memory for new bdt classifier\n");
        return NULL;
    }

    memset(res, 0, sizeof(bdt_t));

    if (!(res->categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate memory for categories array for bdt classifier.\n");
        free_bdt(res);
        return NULL;
    }

    memset(res->categories, 0, cat_num * sizeof(uint8_t));

    for (uint8_t cat = 0; cat < cat_num; ++cat) {
        res->categories[cat] = categories[cat];
    }

    if (!(res->class_scratch_space = malloc(class_num * sizeof(double)))) {
        printf("Failed to allocate memory for bdt class scratch space.\n");
        free_bdt(res);
        return NULL;
    }

    memset(res->class_scratch_space, 0, class_num * sizeof(double));

    if (!(res->branch_scratch_space = malloc(branch_factor * sizeof(double)))) {
        printf("Failed to allocate memory for bdt branch scratch space.\n");
        free_bdt(res);
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


    if (!(res->nodes = malloc(1 * sizeof(bdt_node_t *)))) {
        printf("Failed to allocate memory for bdt nodes array.\n");
        free_bdt(res);
        return NULL;
    }

    memset(res->nodes, 0, 1 * sizeof(bdt_node_t *));

    if (!(res->nodes[0] = create_bdt_node_leaf(res))) {
        printf("Failed to create initial bdt classifier node.\n");
        free_bdt(res);
        return NULL;
    }

    res->nodes[0]->node_id = 0;
    res->nodes_num = 1;

    return res;
}

void bdt_train_single(
        bdt_t * bdt,
        const uint8_t * data,
        const double * label)
{
    node_train_single(
            bdt,
            bdt->nodes[0],
            data,
            label);

    return;
}

void bdt_train_batch(
        bdt_t * bdt,
        const uint8_t * const * datas,
        const double * const * labels,
        size_t dsize)
{
    uint64_t start_time;
    uint64_t end_time;

    printf("BDT training batch on %zu data points.\n", dsize);
    start_time = time(NULL);

    for (size_t d = 0; d < dsize; ++d) {
        bdt_train_single(
                bdt,
                datas[d],
                labels[d]);
    }

    end_time = time(NULL);

    printf("Time spent training: %lu\n", end_time - start_time);

    return;
}

void bdt_predict_class(
        const bdt_t * bdt,
        const uint8_t * data,
        double * labels)
{
    return node_predict_class(
            bdt,
            bdt->nodes[0],
            data,
            labels);
}

void free_bdt(
        bdt_t * bdt)
{
    if (!bdt) {
        return;
    }

    if (bdt->categories) {
        free(bdt->categories);
        bdt->categories = NULL;
    }

    if (bdt->class_scratch_space) {
        free(bdt->class_scratch_space);
        bdt->class_scratch_space = NULL;
    }

    if (bdt->branch_scratch_space) {
        free(bdt->branch_scratch_space);
        bdt->branch_scratch_space = NULL;
    }

    if (bdt->nodes) {
        for (size_t n = 0; n < bdt->nodes_num; ++n) {
            free_bdt_node(bdt->nodes[n]);
            bdt->nodes[n] = NULL;
        }
        free(bdt->nodes);
        bdt->nodes = NULL;
    }

    free(bdt);
    return;
}

bdt_t * bdt_from_file(
        FILE * file,
        uint8_t use_probs)
{
    bdt_t * res;
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
        printf("Failed to allocate memory for categories of bdt classifier being read from file.\n");
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

    if (!(res = malloc(sizeof(bdt_t)))) {
        printf("Failed to allocate memory for bdt classifier being read from file.\n");
        free(categories);
        return NULL;
    }

    memset(res, 0, sizeof(bdt_t));

    res->categories = categories;
    res->cat_num = cat_num;
    res->nodes_num = nodes_num;
    res->class_num = class_num;
    res->branch_factor = branch_factor;
    res->split_threshold = split_threshold;
    res->split_limit = split_limit;
    res->split_number = split_number;
    res->forget_factor = forget_factor;
    res->nb_alpha = nb_alpha;
    res->use_probs = use_probs;

    if (!(res->nodes = malloc(nodes_num * sizeof(bdt_node_t *)))) {
        printf("Failed to allocate memory for bdt node array while reading bdt classifier from file.\n");
        free_bdt(res);
        return NULL;
    }

    for (size_t n = 0; n < nodes_num; ++n) {
        if (!(res->nodes[n] = bdt_node_from_file(
                res,
                file)))
        {
            printf("Failed to read bdt node from file.\n");
            free_bdt(res);
            return NULL;
        }
    }

    return res;
}


bdt_t * bdt_from_file_with_name(
        const char * filename,
        uint8_t use_probs)
{
    FILE * file;
    bdt_t * res;

    if (!(file = fopen(filename, "r"))) {
        printf("Failed to open file %s for reading bdt model.\n", filename);
        return NULL;
    }

    if (!(res = bdt_from_file(
            file,
            use_probs)))
    {
        printf("Failed to read bdt from file: %s.\n", filename);
        fclose(file);
        return NULL;
    }

    fclose(file);
    return res;
}

void bdt_to_file(
        const bdt_t * bdt,
        FILE * file)
{
    fprintf(file, "%u\n", bdt->cat_num);

    for (uint32_t cat = 0; cat < bdt->cat_num; ++cat) {
        fprintf(file, "%hhu\n", bdt->categories[cat]);
    }

    fprintf(file, "%hhu\n", bdt->class_num);
    fprintf(file, "%hhu\n", bdt->branch_factor);
    fprintf(file, "%u\n", bdt->split_threshold);
    fprintf(file, "%u\n", bdt->split_limit);
    fprintf(file, "%u\n", bdt->split_number);
    fprintf(file, "%zu\n", bdt->nodes_num);
    fprintf(file, "%lf\n", bdt->forget_factor);
    fprintf(file, "%lf\n", bdt->nb_alpha);

    for (size_t n = 0; n < bdt->nodes_num; ++n) {
        bdt_node_to_file(
                bdt->nodes[n],
                file);
    }

    return;
}

void bdt_to_file_with_name(
        const bdt_t * bdt,
        const char * filename)
{
    FILE * file;

    if (!(file = fopen(filename, "w"))) {
        printf("Failed to open file %s for writing bdt model.\n", filename);
        return;
    }

    bdt_to_file(
            bdt,
            file);
    return;
}

static bdt_node_t * create_bdt_node_shell(
        const bdt_t * bdt)
{
    bdt_node_t * res;

    if (!(res = malloc(sizeof(bdt_node_t)))) {
        printf("Failed to allocate memory for bdt node.\n");
        return NULL;
    }

    memset(res, 0, sizeof(bdt_node_t));

    res->root_bdt = (bdt_t *) bdt;
    res->children = NULL;
    res->num_trained = 0;

    return res;
}

static bdt_node_t * create_bdt_node_leaf(
        const bdt_t * bdt)
{
    bdt_node_t * res;

    if (!(res = create_bdt_node_shell(bdt))) {
        printf("Failed to create shell bdt node.\n");
        return NULL;
    }

    res->type = LEAF;
    if (!(res->classifier = create_cnbp_with_alpha(
            bdt->class_num,
            bdt->categories,
            bdt->cat_num,
            bdt->nb_alpha,
            bdt->use_probs)))
    {
        printf("Failed to create cnbp for bdt node.\n");
        free_bdt_node(res);
        return NULL;
    }

    return res;
}

static void node_train_single(
        bdt_t * bdt,
        bdt_node_t * node,
        const uint8_t * data,
        const double * label)
{
    double child_error;
    double min_error;
    uint8_t min_error_index;
    double * expected;
    double * branch_labels;

    if (node->type == LEAF) {

        if (node->num_trained >= bdt->split_threshold && bdt->split_number < bdt->split_limit) {

            printf("Splitting Node with node_id: %u.\n", node->node_id);

            if (!(node->children = malloc(bdt->branch_factor * sizeof(uint32_t)))) {
                printf("Failed to allocate array for children indices.\n");
                return;
            }

            if (!(bdt->nodes = realloc(
                    bdt->nodes,
                    (bdt->nodes_num + bdt->branch_factor) * sizeof(bdt_node_t *))))
            {
                printf("Failed to reallocate the nodes array of bdt.\n");
                return;
            }

            for (uint8_t child = 0; child < bdt->branch_factor; ++child) {
                node->children[child] = bdt->nodes_num + child;

                if (!(bdt->nodes[bdt->nodes_num + child] = create_bdt_node_shell(bdt))) {
                    printf("Failed to create new bdt node shell for node split.\n");
                    return;
                }

                bdt->nodes[bdt->nodes_num + child]->node_id = bdt->nodes_num + child;
                bdt->nodes[bdt->nodes_num + child]->type = LEAF;

                if (!(bdt->nodes[bdt->nodes_num + child]->classifier = copy_cnbp(node->classifier))) {
                    printf("Failed to copy cnbp of bdt node for node split.\n");
                    return;
                }

                cnbp_forget(
                        bdt->nodes[bdt->nodes_num + child]->classifier,
                        bdt->forget_factor);
            }

            free_cnbp(node->classifier);

            if (!(node->classifier = create_cnbp_with_alpha(
                    bdt->branch_factor,
                    bdt->categories,
                    bdt->cat_num,
                    bdt->nb_alpha,
                    bdt->use_probs)))
            {
                printf("Failed to create new classifier for branch node.\n");
                return;
            }

            node->type = BRANCH;
            node->num_trained = 0;
            bdt->nodes_num += bdt->branch_factor;
            bdt->split_number++;

            node_train_single(
                    bdt,
                    node,
                    data,
                    label);
        }
        else {
            cnbp_train_single(
                    node->classifier,
                    data,
                    label);
            node->num_trained++;
            return;
        }
    }
    else if (node->type == BRANCH) {

//        printf("Training Branch Node, node_id: %u\n", node->node_id);

        for (uint8_t child = 0; child < bdt->branch_factor; ++child) {

            node_predict_class(
                    bdt,
                    bdt->nodes[node->children[child]],
                    data,
                    bdt->class_scratch_space);
            expected = bdt->class_scratch_space;
            child_error = 0.0;

            for (uint8_t cl = 0; cl < bdt->class_num; ++cl) {
                child_error += fabs(expected[cl] - label[cl]);
            }

            child_error /= 2;

            if (!child || child_error < min_error) {
                min_error = child_error;
                min_error_index = child;
            }

        }

        if (!(branch_labels = malloc(bdt->branch_factor * sizeof(double)))) {
            printf("Failed to allocate array for branch labels during training.\n");
            return;
        }

        for (uint8_t b = 0; b < bdt->branch_factor; ++b) {
            branch_labels[b] = 0.0;
        }

        branch_labels[min_error_index] = 1.0;

        cnbp_train_single(
                node->classifier,
                data,
                branch_labels);

        free(branch_labels);

        node_train_single(
                bdt,
                bdt->nodes[node->children[min_error_index]],
                data,
                label);

        node->num_trained++;
        return;
    }

    return;
}

static void node_predict_class(
        const bdt_t * bdt,
        bdt_node_t * node,
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
                bdt->branch_scratch_space);
        branch_expected = bdt->branch_scratch_space;

        for (uint8_t b = 0; b < bdt->branch_factor; ++b) {

            if (!b || branch_expected[b] > max) {
                max = branch_expected[b];
                max_index = b;
            }

        }

        node_predict_class(
                bdt,
                bdt->nodes[node->children[max_index]],
                data,
                labels);
    }
    else {
        printf("Unrecognized bdt NODE TYPE: %u.\n", node->type);
    }

    return;
}

static void free_bdt_node(
        bdt_node_t * node)
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

static bdt_node_t * bdt_node_from_file(
        const bdt_t * bdt,
        FILE * file)
{
    bdt_node_t * res;

    if (!(res = create_bdt_node_shell(bdt))) {
        printf("Failed to create bdt node shell.\n");
        return NULL;
    }

    fscanf(file, "%u\n", &(res->type));
    fscanf(file, "%u\n", &(res->node_id));
    fscanf(file, "%u\n", &(res->num_trained));

    if (res->type == BRANCH) {

        if (!(res->children = malloc(bdt->branch_factor * sizeof(uint32_t)))) {
            printf("Failed to allocate children array for bdt node while reading from file.\n");
            free_bdt_node(res);
            return NULL;
        }

        memset(res->children, 0, bdt->branch_factor * sizeof(uint32_t));

        for (uint8_t child = 0; child < bdt->branch_factor; ++child) {
            fscanf(file, "%u\n", &(res->children[child]));
        }
    }

    if (!(res->classifier = cnbp_from_file_with_params(
            file,
            res->type == LEAF ? bdt->class_num : bdt->branch_factor,
            bdt->categories,
            bdt->cat_num,
            bdt->nb_alpha,
            bdt->use_probs)))
    {
        printf("Failed to read cnbp from file for bdt node.\n");
        free_bdt_node(res);
    }

    return res;
}

static void bdt_node_to_file(
        const bdt_node_t * node,
        FILE * file)
{
    fprintf(file, "%u\n", (uint32_t) node->type);
    fprintf(file, "%u\n", node->node_id);
    fprintf(file, "%u\n", node->num_trained);

    if (node->type == BRANCH) {

        for (uint8_t child = 0; child < node->root_bdt->branch_factor; ++child) {
            fprintf(file, "%u\n", node->children[child]);
        }

    }

    cnbp_to_file_no_params(
            node->classifier,
            file);
    return;
}
