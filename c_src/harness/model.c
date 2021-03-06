#include "model.h"

#include "../models/cnbp/cnbp.h"
#include "../models/bdt/bdt.h"

#include <string.h>
#include <stdlib.h>

#define POS_CHAR_BUFSIZE 512
#define POS_UINT8_BUFSIZE 128

/** RANDOM model callbacks **/
typedef struct random_eval_model {
    uint8_t class_num;
} rand_t;

static void * random_new_model(
        const char ** args);
static void * random_read_model(
        const char * filename,
        const char ** args);
static void random_write_model(
        const void * model,
        const char * filename);
static void random_train_model(
        void * model,
        const uint8_t * data,
        const double * labels);
static void random_run_model(
        const void * model,
        const uint8_t * data,
        double * labels);
static void random_free_model(
        void * model);

/** NB (Naive Bayes) Model callbacks **/
static void * nb_new_model(
        const char ** args);
static void * nb_read_model(
        const char * filename,
        const char ** args);
static void nb_write_model(
        const void * model,
        const char * filename);
static void nb_train_model(
        void * model,
        const uint8_t * data,
        const double * labels);
static void nb_run_model(
        const void * model,
        const uint8_t * data,
        double * labels);
static void nb_free_model(
        void * model);

/** BDT (Bayesian Delegation Tree) Model callbacks **/
static void * bdt_new_model(
        const char ** args);
static void * bdt_read_model(
        const char * filename,
        const char ** args);
static void bdt_write_model(
        const void * model,
        const char * filename);
static void bdt_train_model(
        void * model,
        const uint8_t * data,
        const double * labels);
static void bdt_run_model(
        const void * model,
        const uint8_t * data,
        double * labels);
static void bdt_free_model(
        void * model);

static uint32_t train_model_on_file(
        em_t * model,
        const char * filename,
        const double * labels);


em_t * model(
        const char *model_type)
{
    em_t * res;

    if (!(res = malloc(sizeof(em_t)))) {
        printf("Failed to allocate memory for model.\n");
        return NULL;
    }

    if (!strcmp(model_type, RANDOM)) {
        res->model = NULL;
        res->new_model = &(random_new_model);
        res->read_model = &(random_read_model);
        res->write_model = &(random_write_model);
        res->train_model = &(random_train_model);
        res->run_model = &(random_run_model);
        res->free_model = &(random_free_model);
    }
    else if (!strcmp(model_type, NB)) {
        res->model = NULL;
        res->new_model = &(nb_new_model);
        res->read_model = &(nb_read_model);
        res->write_model = &(nb_write_model);
        res->train_model = &(nb_train_model);
        res->run_model = &(nb_run_model);
        res->free_model = &(nb_free_model);

    }
    else if (!strcmp(model_type, BDT)) {
        res->model = NULL;
        res->new_model = &(bdt_new_model);
        res->read_model = &(bdt_read_model);
        res->write_model = &(bdt_write_model);
        res->train_model = &(bdt_train_model);
        res->run_model = &(bdt_run_model);
        res->free_model = &(bdt_free_model);
    }
    else {
        printf("Unrecognized evaluation model type.\n");
        free_model(res);
        return NULL;
    }

    return res;
}

void new_model(
        em_t * model,
        const char ** args)
{
    if (!(model->model = model->new_model(args))) {
        printf("Failed to create new model.\n");
        return;
    }

    return;
}

void free_model(em_t * model) {

    if (!model) {
        return;
    }

    if (model->model) {
        model->free_model(model->model);
        model->model = NULL;
    }

    free(model);
}

em_t * load_model(
        const char * model_type,
        uint32_t model_version,
        const char * directory,
        const char ** args)
{
    em_t * res;
    char * filename;
    size_t filename_size;

    if (!(res = model(
            model_type)))
    {
        printf("Failed to create model of type: %s\n", model_type);
        return NULL;
    }

    if (!model_version) {
        res->model = res->new_model(args);
    }
    else {
        filename_size = snprintf(NULL, 0, "%s/%s%u.model", directory, model_type, model_version) + 1;

        if (!(filename = malloc(filename_size * sizeof(char)))) {
            printf("Failed to allocate memory for filename.\n");
            free_model(res);
            return NULL;
        }

        memset(filename, 0, filename_size * sizeof(char));
        snprintf(filename, filename_size, "%s/%s%u.model", directory, model_type, model_version);

        if (!(res->model = res->read_model(
                (const char *) filename,
                args)))
        {
            printf("Failed to read model from file: %s.\n", filename);
            free_model(res);
            free(filename);
            return NULL;
        }
    }

    return res;
}

void store_model(
        em_t * model,
        const char * model_type,
        uint32_t model_version,
        const char * directory)
{
    size_t filename_size;
    char * filename;

    if (!model->model) {
        printf("Attempting to store model, but pointer to model struct is NULL.\n");
        return;
    }

    filename_size = snprintf(NULL, 0, "%s/%s%u.model", directory, model_type, model_version) + 1;

    if (!(filename = malloc(filename_size * sizeof(char)))) {
        printf("Failed to allocate memory for model filename.\n");
        return;
    }

    memset(filename, 0, filename_size * sizeof(char));
    snprintf(filename, filename_size, "%s/%s%u.model", directory, model_type, model_version);

    model->write_model(
            model->model,
            filename);

    free(filename);
}

void train_model(
        em_t * model,
        const char * directory)
{
    char * filename;
    size_t fsize;
    double labels[3];
    uint32_t num_trained = 0;
    uint32_t total_trained = 0;

    fsize = snprintf(NULL, 0, "%s/white_wins.games", directory) + 1;

    if (!(filename = malloc(fsize * sizeof(char)))) {
        printf("Failed to allocate memory for filename with white wins.\n");
        return;
    }

    memset(filename, 0, fsize * sizeof(char));
    snprintf(filename, fsize, "%s/white_wins.games", directory);
    labels[0] = 1.0;
    labels[1] = 0.0;
    labels[2] = 0.0;

    num_trained = train_model_on_file(
            model,
            filename,
            labels);

    printf("Model trained on %u White Wins.\n", num_trained);
    total_trained += num_trained;

    free(filename);

    fsize = snprintf(NULL, 0, "%s/black_wins.games", directory) + 1;

    if (!(filename = malloc(fsize * sizeof(char)))) {
        printf("Failed to allocate memory for filename with black wins.\n");
        return;
    }

    memset(filename, 0, fsize * sizeof(char));
    snprintf(filename, fsize, "%s/black_wins.games", directory);
    labels[0] = 0.0;
    labels[1] = 1.0;
    labels[2] = 0.0;

    num_trained = train_model_on_file(
            model,
            filename,
            labels);

    printf("Model trained on %u Black Wins.\n", num_trained);
    total_trained += num_trained;

    free(filename);

    fsize = snprintf(NULL, 0, "%s/draws.games", directory) + 1;

    if (!(filename = malloc(fsize * sizeof(char)))) {
        printf("Failed to allocate memory for filename with draws.\n");
        return;
    }

    memset(filename, 0, fsize * sizeof(char));
    snprintf(filename, fsize, "%s/draws.games", directory);
    labels[0] = 0.0;
    labels[1] = 0.0;
    labels[2] = 1.0;

    num_trained = train_model_on_file(
            model,
            filename,
            labels);

    printf("Model trained on %u Draws.\n", num_trained);
    total_trained += num_trained;
    printf("Total training data points: %u.\n", total_trained);

    free(filename);
}


static void * random_new_model(
        const char ** args) {
    rand_t * res;

    if (!(res = malloc(sizeof(rand_t)))) {
        printf("Failed to allocate memory for random evaluation model.\n");
        return NULL;
    }

    if (!(*args)) {
        printf("Random evaluation model missing argument for number of classes.\n");
        free(res);
        return NULL;
    }

    res->class_num = (uint8_t)atoi(args[0]);
    return (void *) res;
}

static void * random_read_model(
        const char * filename,
        const char ** args) {
    printf("Cannot read random evaluation model from file: %s (or any file for that matter)\n.", filename);
    return NULL;
}

static void random_write_model(
        const void * model,
        const char * filename) {
    printf("Cannot write random evaluation model to file: %s (or any file for that matter)\n.", filename);
    return;
}

static void random_train_model(
        void * model,
        const uint8_t * data,
        const double * labels) {
    printf("Cannot train random evaluation model\n.");
    return;
}

static void random_run_model(
        const void * model,
        const uint8_t * data,
        double * labels)
{
    rand_t * emod = (rand_t *) model;

    for (uint8_t cl = 0; cl < emod->class_num; ++cl) {
        labels[cl] = 1.0 / emod->class_num;
    }

    return;
}

static void random_free_model(
        void * model)
{
    if (model) {
        free(model);
    }

    return;
}


static void * nb_new_model(
        const char ** args)
{
    cnbp_t * res;
    uint8_t class_num;
    uint8_t cat_num;
    uint8_t *categories;
    double alpha;
    uint8_t use_probs;
    int at = 0;

    if(!args[at]) {
        printf("NB model missing class_num parameter.\n");
        return NULL;
    }
    else {
        class_num = (uint8_t) atoi(args[at]);
        ++at;
    }

    if(!args[at]) {
        printf("NB model missing cat_num parameter.\n");
        return NULL;
    }
    else {
        cat_num = (uint32_t) atoi(args[at]);
        ++at;
    }

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate categories array while creating NB model.\n");
        return NULL;
    }

    for (uint32_t cat = 0; cat < cat_num; ++cat) {

        if (!args[at+cat]) {
            printf("NB model missing expected number of categories.\n");
            free(categories);
            return NULL;
        }
        else {
            categories[cat] = (uint8_t) atoi(args[at + cat]);
        }

    }

    at += cat_num;

    if (!args[at]) {
        printf("NB model missing Laplace smoothing alpha parameter.\n");
        free(categories);
        return NULL;
    }
    else {
        alpha = atof(args[at]);
        ++at;
    }

    if (!args[at]) {
        printf("NB model missing use_probs parameter.\n");
        free(categories);
        return NULL;
    }
    else {
        use_probs = (uint8_t) atoi(args[at]);
        ++at;
    }

    if (!(res = create_cnbp_with_alpha(
            class_num,
            categories,
            (size_t) cat_num,
            alpha,
            use_probs)))
    {
        printf("Failed to create cnbp model.\n");
        free(categories);
        return NULL;
    }

    free(categories);
    return (void *) res;
}


static void * nb_read_model(
        const char * filename,
        const char ** args)
{
    cnbp_t * res;
    uint8_t use_probs;

    if (!(*args)) {
        printf("NB model to be read from file is missing use_probs argument.\n");
        return NULL;
    }
    else {
        use_probs = atoi(args[0]);
    }

    if (!(res = cnbp_from_file_with_name(
            filename,
            use_probs))) {
        printf("Failed to read NB model from file.\n");
        return NULL;
    }

    return (void *) res;
}

static void nb_write_model(
        const void * model,
        const char * filename)
{
    return cnbp_to_file_with_name(
            (cnbp_t *) model,
            filename);
}

static void nb_train_model(
        void * model,
        const uint8_t * data,
        const double * labels)
{
    return cnbp_train_single(
            (cnbp_t *) model,
            data,
            labels);
}

static void nb_run_model(
        const void * model,
        const uint8_t * data,
        double * labels)
{
    return cnbp_predict_class(
            (cnbp_t *) model,
            data,
            labels);
}

static void nb_free_model(
        void * model)
{
    return free_cnbp((cnbp_t *) model);
}


static void * bdt_new_model(
        const char ** args)
{
    bdt_t * res;
    uint8_t class_num;
    uint8_t cat_num;
    uint8_t *categories;
    uint8_t branch_factor;
    uint32_t split_threshold;
    uint32_t split_limit;
    double forget_factor;
    double alpha;
    uint8_t use_probs;
    int at = 0;

    if(!args[at]) {
        printf("BDT model missing class_num parameter.\n");
        return NULL;
    }
    else {
        class_num = (uint8_t) atoi(args[at]);
        ++at;
    }

    if(!args[at]) {
        printf("BDT model missing cat_num parameter.\n");
        return NULL;
    }
    else {
        cat_num = (uint32_t) atoi(args[at]);
        ++at;
    }

    if (!(categories = malloc(cat_num * sizeof(uint8_t)))) {
        printf("Failed to allocate categories array while creating BDT model.\n");
        return NULL;
    }

    for (uint32_t cat = 0; cat < cat_num; ++cat) {

        if (!args[at+cat]) {
            printf("BDT model missing expected number of categories.\n");
            free(categories);
            return NULL;
        }
        else {
            categories[cat] = (uint8_t) atoi(args[at + cat]);
        }

    }

    at += cat_num;

    if (!args[at]) {
        printf("BDT model missing branch_factor parameter.\n");
        return NULL;
    }
    else {
        branch_factor = (uint8_t) atoi(args[at]);
        ++at;
    }

    if (!args[at]) {
        printf("BDT model missing split_threshold parameter.\n");
        return NULL;
    }
    else {
        split_threshold = (uint32_t) atoi(args[at]);
        ++at;
    }

    if (!args[at]) {
        printf("BDT model missing split_limit parameter.\n");
        return NULL;
    }
    else {
        split_limit = (uint32_t) atoi(args[at]);
        ++at;
    }

    if (!(args[at])) {
        printf("BDT model missing forget_factor parameter.\n");
        return NULL;
    }
    else {
        forget_factor = atof(args[at]);
        ++at;
    }

    if (!args[at]) {
        printf("BDT model missing Laplace smoothing alpha parameter.\n");
        free(categories);
        return NULL;
    }
    else {
        alpha = atof(args[at]);
        ++at;
    }

    if (!args[at]) {
        printf("BDT model missing use_probs parameter.\n");
        free(categories);
        return NULL;
    }
    else {
        use_probs = (uint8_t) atoi(args[at]);
        ++at;
    }

    if (!(res = create_bdt(
            categories,
            (size_t) cat_num,
            class_num,
            branch_factor,
            split_threshold,
            split_limit,
            forget_factor,
            alpha,
            use_probs)))
    {
        printf("Failed to create bdt model.\n");
        free(categories);
        return NULL;
    }

    free(categories);
    return (void *) res;
}

static void * bdt_read_model(
        const char * filename,
        const char ** args)
{
    bdt_t * res;
    uint8_t use_probs;

    if (!(*args)) {
        printf("BDT model to be read from file is missing use_probs argument.\n");
        return NULL;
    }
    else {
        use_probs = atoi(args[0]);
    }

    if (!(res = bdt_from_file_with_name(
            filename,
            use_probs))) {
        printf("Failed to read BDT model from file.\n");
        return NULL;
    }

    return (void *) res;
}

static void bdt_write_model(
        const void * model,
        const char * filename)
{
    return bdt_to_file_with_name(
            (bdt_t *) model,
            filename);
}

static void bdt_train_model(
        void * model,
        const uint8_t * data,
        const double * labels)
{
    return bdt_train_single(
            (bdt_t *) model,
            data,
            labels);
}

static void bdt_run_model(
        const void * model,
        const uint8_t * data,
        double * labels)
{
    return bdt_predict_class(
            (bdt_t *) model,
            data,
            labels);
}

static void bdt_free_model(
        void * model)
{
    return free_bdt(
            (bdt_t *) model);
}

static uint32_t train_model_on_file(
        em_t * model,
        const char * filename,
        const double * labels)
{
    FILE * file;
    uint8_t data_array[POS_UINT8_BUFSIZE];
    char position_string[POS_CHAR_BUFSIZE];
    char * strval;
    size_t at = 0;
    uint32_t lines = 0;

    memset(position_string, 0, sizeof(position_string));

    if (!(file = fopen(filename, "r"))) {
        printf("Failed to open file: %s\n", filename);
        return 0;
    }

    while (fgets((char *)position_string, POS_CHAR_BUFSIZE, file)) {

        at = 0;

        if ((strval = strtok((char *)position_string, " "))) {
            data_array[at] = (uint8_t) atoi(strval);
            ++at;
        }

        while ((strval = strtok(NULL , " "))) {
            data_array[at] = (uint8_t) atoi(strval);
            ++at;
        }

        model->train_model(
                model->model,
                (const uint8_t *) data_array,
                (const double *) labels);

        ++lines;
    }

    fclose(file);

    return lines;
}
