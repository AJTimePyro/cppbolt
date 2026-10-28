#include <cstdio>
#include <cstdlib>

int main() {
    void* impossible = malloc(static_cast<size_t>(-1));
    printf("%d\n", impossible == nullptr);
    free(impossible);
    return 0;
}
