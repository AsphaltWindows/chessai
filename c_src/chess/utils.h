#ifndef UTILS_H
#define UTILS_H

#include "types.h"

uint64_t position_hash_index(
        const position_t * const pos,
        uint64_t prime);

inline piece_t piece_with_color(
        piecetype_t pt,
        color_t c);

inline piecetype_t type_of_piece(
        piece_t p);

inline color_t color_of_piece(
        piece_t p);

inline square_t square(
        file_t f,
        rank_t r);

inline file_t square_file(
        square_t s);

inline rank_t square_rank(
        square_t s);

#endif //UTILS_H
