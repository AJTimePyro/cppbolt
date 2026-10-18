#include <cstdio>

enum Priority {
    LOW = 10,
    MEDIUM = 25,
    HIGH = 50,
    CRITICAL = 100
};

int main() {
    Priority p1 = LOW;
    Priority p2 = HIGH;
    Priority p3 = CRITICAL;

    std::printf("p1: %d\n", (int)p1);
    std::printf("p2: %d\n", (int)p2);
    std::printf("p3: %d\n", (int)p3);
    std::printf("sum: %d\n", (int)p1 + (int)p2);
    std::printf("cmp: %d\n", p2 > p1 ? 1 : 0);
    return 0;
}
