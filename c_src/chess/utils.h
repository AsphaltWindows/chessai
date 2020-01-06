#ifndef UTILS_H
#define UTILS_H

#include "position.h"

uint64_t position_hash_index(
        const position_t * const pos,
        uint64_t prime);

#endif //UTILS_H
