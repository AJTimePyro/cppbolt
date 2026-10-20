#include <cstdio>
#include <cstdlib>

struct Block {
    char* ptr;
    int size;
    int id;
};

int main() {
    unsigned int prng = 4242;
    unsigned int checksum = 0;
    Block blocks[40];
    int count = 0;

    for (int round = 0; round < 120; round++) {
        int to_alloc = (prng % 3) + 1;
        prng = (prng * 1103515245 + 12345) & 0x7fffffff;

        for (int k = 0; k < to_alloc && count < 40; k++) {
            int sz = ((prng % 24) + 1) * 16;
            prng = (prng * 1103515245 + 12345) & 0x7fffffff;
            int id = round * 100 + k;

            char* p = (char*)malloc(sz);
            if (!p) {
                printf("Alloc failed!\n");
                return 1;
            }

            if (((unsigned long)p) % 8 != 0) {
                printf("Unaligned pointer: %p\n", p);
                return 2;
            }

            for (int j = 0; j < count; j++) {
                char* p_start = p;
                char* p_end = p + sz;
                char* o_start = blocks[j].ptr;
                char* o_end = blocks[j].ptr + blocks[j].size;
                if (!(p_end <= o_start || p_start >= o_end)) {
                    printf("Overlap detected between %p and %p!\n", p, blocks[j].ptr);
                    return 3;
                }
            }

            for (int b = 0; b < sz; b++) {
                char val = (char)((id + b) & 0xff);
                p[b] = val;
                checksum = (checksum * 131 + (unsigned char)val) & 0x7fffffff;
            }

            blocks[count].ptr = p;
            blocks[count].size = sz;
            blocks[count].id = id;
            count++;
        }

        if (count > 0 && (prng % 2 == 0 || count >= 36)) {
            int idx = prng % count;
            prng = (prng * 1103515245 + 12345) & 0x7fffffff;

            Block b = blocks[idx];
            for (int byte_i = 0; byte_i < b.size; byte_i++) {
                char expected = (char)((b.id + byte_i) & 0xff);
                if (b.ptr[byte_i] != expected) {
                    printf("Pattern mismatch at offset %d!\n", byte_i);
                    return 4;
                }
            }

            free(b.ptr);
            blocks[idx] = blocks[count - 1];
            count--;
        }
    }

    for (int i = 0; i < count; i++) {
        Block b = blocks[i];
        for (int byte_i = 0; byte_i < b.size; byte_i++) {
            char expected = (char)((b.id + byte_i) & 0xff);
            if (b.ptr[byte_i] != expected) {
                printf("Final pattern mismatch at offset %d!\n", byte_i);
                return 5;
            }
        }
        free(b.ptr);
    }

    printf("Allocator stress: OK, checksum: %u\n", checksum);
    return 0;
}
