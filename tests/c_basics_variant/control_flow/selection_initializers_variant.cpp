#include <cstdio>

int main() {
    if (int a = 15, b = 25; a + b > 30) {
        std::printf("sum: %d\n", a + b);
    }

    if (int v = 0; v != 0) {
        std::printf("nonzero\n");
    } else if (int w = 7; w == 7) {
        std::printf("w: %d\n", w);
    } else {
        std::printf("other\n");
    }

    switch (int val = 100; val / 25) {
        case 3: std::printf("val 75\n"); break;
        case 4: std::printf("val 100\n"); break;
        default: std::printf("val other\n"); break;
    }

    return 0;
}
