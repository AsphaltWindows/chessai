#include "player.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

/** We select the move that gives the highest expected score (win = 1 loss = 0, draw = 1)**/
static int wld_score_compare(color_t color, const double * first, const double * second);
/** If positions is available with higher win % than loss % we select that of those positions where win % is the highest
 * If there are no positions such that win % > loss % we select the position with the highest draw %.
 **/
static int wld_win_draw_compare(color_t color, const double * first, const double * second);

player_t * player(
        em_t * eval_model,
        const char * compare_type,
        color_t color)
{
    player_t * res;

    if (!(res = malloc(sizeof(res)))) {
        printf("Failed to allocate memory for player.\n");
        return NULL;
    }

    res->eval_model = eval_model;
    res->color = color;

    if (!strcmp(compare_type, WLD_SCORE)) {
        res->compare_moves = &wld_score_compare;
    }
    else if (!strcmp(compare_type, WLD_WIN_DRAW)) {
        res->compare_moves = &wld_win_draw_compare;
    }
    else {
        printf("Invalid move comparison type provided: %s.\n", compare_type);
        free_player(res);
        return NULL;
    }

    return res;
}

void free_player(
        player_t * player)
{
    if (!player) {
        return;
    }

    if (player->eval_model) {
        free_model(player->eval_model);
        player->eval_model = NULL;
    }

    free(player);
    return;
}

uint32_t select_move(
        const player_t * player,
        const uint8_t * const * positions,
        size_t pos_num)
{
    uint32_t candidate_indices[1024];
    uint32_t num_candidates = 0;
    double best[3];
    double next[3];
    size_t at = 0;
    int comp_res;

    while (at < pos_num) {
        player->eval_model->run_model(player->eval_model->model, positions[at], (double *) next);

        if (!at || ((comp_res = player->compare_moves(
                player->color,
                best,
                next)) == -1))
        {
            num_candidates = 1;
            candidate_indices[0] = at;
            memcpy(best, next, 3 * sizeof(double));
        }
        else if (comp_res == 0) {
            candidate_indices[num_candidates] = at;
            ++num_candidates;
        }

        ++at;
    }

    if (num_candidates == 1) {
        return candidate_indices[0];
    }
    else {
        return candidate_indices[rand() % num_candidates];
    }

}

void switch_color(
        player_t * player) {
    player->color = player->color == WHITE ? BLACK : WHITE;
    return;
}

static int wld_score_compare(
        color_t color,
        const double * first,
        const double * second) {
    int w_idx = color == WHITE ? 0 : 1;
    int d_idx = 2;
    double score_first = first[w_idx] + (first[d_idx] / 2);
    double score_second = second[w_idx] + (second[d_idx] / 2);

    if (score_first == score_second) {
        return 0;
    }
    else if (score_first < score_second) {
        return -1;
    }
    else {
        return 1;
    }
}

static int wld_win_draw_compare(
        color_t color,
        const double * first,
        const double * second) {
    int w_idx = color == WHITE ? 0 : 1;
    int l_idx = color == WHITE ? 1 : 0;
    int d_idx = 2;

    if (first[w_idx] > first[l_idx]) {

        if (second[w_idx] <= second[l_idx]) {
            return 1;
        }
        else if (first[w_idx] > second[w_idx]){
            return 1;
        }
        else if (first[w_idx] == second[w_idx]) {
            return 0;
        }
        else {
            return -1;
        }
    }
    else {
        if (second[w_idx] > second[l_idx]) {
            return -1;
        }
        else if (first[d_idx] > second[d_idx]) {
            return 1;
        }
        else if (first[d_idx] == second[d_idx]) {
            return 0;
        }
        else {
            return -1;
        }
    }

}
