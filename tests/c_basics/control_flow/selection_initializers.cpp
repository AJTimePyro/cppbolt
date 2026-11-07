#include <cstdio>

struct Guard {
    int id;
    Guard(int i) : id(i) {
        std::printf("Guard(%d)\n", id);
    }
    ~Guard() {
        std::printf("~Guard(%d)\n", id);
    }
};

int main() {
    if (int x = 42; x > 0) {
        std::printf("x: %d\n", x);
    }

    if (int y = -5; y > 0) {
        std::printf("y pos: %d\n", y);
    } else {
        std::printf("y neg: %d\n", y);
    }

    switch (int s = 2; s) {
        case 1: std::printf("s: 1\n"); break;
        case 2: std::printf("s: 2\n"); break;
        default: std::printf("s: def\n"); break;
    }

    int count = 0;
    if (count += 10; count > 5) {
        std::printf("count: %d\n", count);
    }

    {
        if (Guard g(100); g.id > 50) {
            std::printf("inside if\n");
        }
        std::printf("after if\n");
    }

    return 0;
}
