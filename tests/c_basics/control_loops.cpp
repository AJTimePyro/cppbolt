#include <cstdio>

int main() {
    int sum = 0;
    for (int i = 0; i < 5; ++i) {
        sum += i;
    }
    std::printf("for sum: %d\n", sum);

    int count = 0;
    int w = 10;
    while (w > 0) {
        w -= 3;
        count++;
    }
    std::printf("while count: %d, final w: %d\n", count, w);

    int d = 0;
    do {
        d += 2;
    } while (d < 6);
    std::printf("do-while d: %d\n", d);

    int break_sum = 0;
    for (int i = 0; i < 100; ++i) {
        if (i % 2 == 0) continue;
        if (i > 7) break;
        break_sum += i;
    }
    std::printf("break_sum: %d\n", break_sum);
    return 0;
}
