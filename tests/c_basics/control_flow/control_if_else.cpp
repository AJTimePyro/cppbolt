#include <cstdio>

void check(int x) {
    if (x > 10) {
        std::printf("%d is greater than 10\n", x);
    } else if (x == 10) {
        std::printf("%d is exactly 10\n", x);
    } else {
        std::printf("%d is less than 10\n", x);
    }
}

int main() {
    check(5);
    check(10);
    check(15);
    return 0;
}
