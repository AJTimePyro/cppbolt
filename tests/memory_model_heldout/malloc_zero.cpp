#include <cstdio>
#include <cstdlib>

int main() {
    void* p = malloc(0);
    void* q = malloc(0);
    if (!p || !q) {
        printf("malloc(0) returned null\n");
        return 1;
    }
    if (((unsigned long)p) % 8 != 0 || ((unsigned long)q) % 8 != 0) {
        printf("malloc(0) not 8-byte aligned\n");
        return 2;
    }
    free(p);
    free(q);

    void* one = malloc(1);
    if (!one) {
        printf("malloc(1) returned null\n");
        return 3;
    }
    free(one);
    printf("malloc(0) twice: non-null aligned; malloc(1): OK\n");
    return 0;
}
