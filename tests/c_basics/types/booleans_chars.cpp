#include <cstdio>

int main() {
    bool t = true;
    bool f = false;
    char c = 'Z';

    std::printf("t && f: %d\n", (t && f) ? 1 : 0);
    std::printf("t || f: %d\n", (t || f) ? 1 : 0);
    std::printf("!f: %d\n", (!f) ? 1 : 0);
    std::printf("char: %c, ascii: %d\n", c, (int)c);

    int val = (c == 'Z') ? 100 : 200;
    std::printf("ternary: %d\n", val);
    return 0;
}
