#include <cstdio>

int main() {
    char c1 = 'A';
    char c2 = 'B';
    bool t = true;
    bool f = false;

    std::printf("char_add: %c\n", c1 + 2);
    std::printf("char_diff: %d\n", c2 - c1);
    std::printf("bool_and: %d\n", (t && f) ? 1 : 0);
    std::printf("bool_or: %d\n", (t || f) ? 1 : 0);
    std::printf("bool_not: %d\n", (!f) ? 1 : 0);
    std::printf("cond: %d\n", (c1 < c2) ? 100 : 200);
    return 0;
}
