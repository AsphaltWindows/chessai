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
