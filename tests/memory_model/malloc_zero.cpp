#include <cstdio>
#include <cstdlib>

int main() {
    void* p = malloc(0);
    if (!p) {
        printf("malloc(0) returned null\n");
        return 1;
    }
    if (((unsigned long)p) % 8 != 0) {
        printf("malloc(0) not 8-byte aligned\n");
        return 2;
    }
    printf("malloc(0): non-null aligned pointer obtained\n");
    free(p);
    printf("free(malloc(0)): OK\n");
    return 0;
}
