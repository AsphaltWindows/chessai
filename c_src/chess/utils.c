#include "utils.h"

#include "murmur3.h"

#define HASH_SIZE 16
#define HASH_SEED 0

uint64_t position_hash_index(
        const position_t * const pos_ptr,
        uint64_t prime)
{
    unsigned char hash[HASH_SIZE];
    MurmurHash3_x64_128(
            (const void *) pos_ptr,
            sizeof(position_t),
            HASH_SEED,
            (void *) hash);

    return (((uint64_t *)hash)[0] ^ ((uint64_t *)hash)[1]) % prime;
}

inline piece_t piece_with_color(
        piecetype_t pt,
        color_t c)
{
    return (piece_t) (c << 3 | pt);
}

inline piecetype_t type_of_piece(
        piece_t p)
{
    return (piecetype_t) (p & 7);
}

inline color_t color_of_piece(
        piece_t p)
{
    return (color_t) (((unsigned int) p) >> 3);
}

inline square_t square(
        file_t f,
        rank_t r)
{
    return (square_t) ((f << 3) | r);
}

inline file_t square_file(
        square_t s);
{
    return (file_t) (((unsigned int) s) >> 3);
}

inline rank_t square_rank(
        square_t s)
{
    return (rank_t) (s & 7);
}

