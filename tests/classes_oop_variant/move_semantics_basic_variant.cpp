#include <cstdio>

struct PairData {
    int a;
    int b;
};

PairData make_data(int x, int y) {
    return PairData{x * 2, y * 3};
}

void print_pair(PairData&& p) {
    printf("pair: %d, %d\n", p.a, p.b);
}

int main() {
    print_pair(make_data(5, 7));
    PairData&& ref = make_data(10, 20);
    printf("bound: %d, %d\n", ref.a, ref.b);
    return 0;
}
