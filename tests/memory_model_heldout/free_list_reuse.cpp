#include <cstdio>
#include <cstdlib>

int main() {
    void* initial = malloc(128);
    free(initial);

    int count = 0;
    for (int i = 0; i < 500000; ++i) {
        void* p = malloc(128);
        if (p == initial) {
            count++;
        }
        free(p);
    }

    printf("Reuses: %d / 500000\n", count);
    return 0;
}
