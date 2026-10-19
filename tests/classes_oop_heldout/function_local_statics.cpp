#include <cstdio>

int tick(int step) {
    static int value = 100;
    value += step;
    return value;
}

int parity() {
    static int calls = 0;
    calls++;
    return calls % 2;
}

int main() {
    for (int i = 0; i < 3; ++i) {
        printf("tick %d = %d\n", i, tick(i + 1));
    }
    int p1 = parity();
    int p2 = parity();
    int p3 = parity();
    printf("parity %d %d %d\n", p1, p2, p3);
    return 0;
}
