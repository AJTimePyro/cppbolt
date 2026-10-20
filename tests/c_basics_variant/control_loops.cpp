#include <cstdio>

int main() {
    int sum = 0;
    for (int i = 0; i < 6; i = i + 1) {
        if (i == 2) continue;
        if (i == 5) break;
        sum = sum + i;
    }
    std::printf("for_sum: %d\n", sum);

    int w = 15;
    int count = 0;
    while (w > 0) {
        w = w - 4;
        count = count + 1;
    }
    std::printf("while_count: %d, w: %d\n", count, w);

    int d = 1;
    do {
        d = d * 3;
    } while (d < 50);
    std::printf("do_while: %d\n", d);
    return 0;
}
