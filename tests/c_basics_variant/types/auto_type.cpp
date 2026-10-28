#include <cstdio>

int main() {
    auto n = 512;
    auto pi = 3.14;
    auto flag = false;
    auto letter = 'Z';
    auto ptr = &n;

    std::printf("n: %d\n", n);
    std::printf("pi: %f\n", pi);
    std::printf("flag: %d\n", flag ? 1 : 0);
    std::printf("letter: %c\n", letter);
    std::printf("deref: %d\n", *ptr);
    return 0;
}
