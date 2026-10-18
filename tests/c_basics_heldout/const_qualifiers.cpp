#include <cstdio>

int main() {
    const int MAX_LIMIT = 256;
    const char SEP = ':';
    const int a = 15;
    const int b = 30;
    const int total = a + b;

    std::printf("limit: %d\n", MAX_LIMIT);
    std::printf("sep: %c\n", SEP);
    std::printf("total: %d\n", total);
    return 0;
}
