#include <cstdio>
#include <cstdlib>

int main() {
    void* initial = malloc(128);
    free(initial);

    int count = 0;
    int i = 0;
    while (i < 500000) {
        void* p = malloc(128);
        if (p == initial) {
            count++;
        }
        free(p);
        i += 1;
    }

    printf("Reuses: %d / 500000\n", count);
    return 0;
}
