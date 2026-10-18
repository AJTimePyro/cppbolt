#include <cstdio>

int main() {
    auto i = 42;
    auto d = 3.14;
    auto* p = &i;
    auto ch = 'A';

    std::printf("auto i: %d\n", i);
    std::printf("auto d: %.2f\n", d);
    std::printf("auto *p: %d\n", *p);
    std::printf("auto ch: %c\n", ch);
    return 0;
}
