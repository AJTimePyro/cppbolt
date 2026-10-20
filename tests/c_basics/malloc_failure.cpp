#include <cstdio>
#include <cstdlib>

int main() {
    void* huge = malloc(300 * 1024 * 1024);
    void* maximum = malloc(0x7fffffff);
    printf("%d %d\n", huge == nullptr, maximum == nullptr);
    free(huge);
    free(maximum);
    return 0;
}
