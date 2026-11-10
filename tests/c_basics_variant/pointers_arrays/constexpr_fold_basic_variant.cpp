#include <iostream>

int main() {
    int buf[((1 << 4) | (3 * 2)) - 2];
    std::cout << sizeof(buf) / sizeof(int) << std::endl;
    int flags[((5 > 2) && (10 != 10)) ? 1 : 3];
    std::cout << sizeof(flags) / sizeof(int) << std::endl;
    return 0;
}
