#include <cstdio>
#include <cstdlib>

int main() {
    void* first_ptr = malloc(64);
    free(first_ptr);

    int reuses = 0;
    for (int i = 0; i < 1000000; ++i) {
        void* p = malloc(64);
        if (p == first_ptr) {
            reuses++;
        }
        free(p);
    }

    printf("Reuses: %d / 1000000\n", reuses);
    return 0;
}
