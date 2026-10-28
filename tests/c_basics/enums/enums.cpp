#include <cstdio>

enum Color {
    RED = 1,
    GREEN = 5,
    BLUE
};

int main() {
    Color c1 = RED;
    Color c2 = GREEN;
    Color c3 = BLUE;

    std::printf("RED: %d\n", c1);
    std::printf("GREEN: %d\n", c2);
    std::printf("BLUE: %d\n", c3);
    std::printf("equal: %d\n", c2 == 5 ? 1 : 0);
    return 0;
}
