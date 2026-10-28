#include <cstdio>

struct IntWrapper {
    int value;

    IntWrapper(int v) : value(v) {}

    operator int() const {
        return value;
    }

    explicit operator double() const {
        return (double)value + 0.5;
    }
};

int add_ten(int val) {
    return val + 10;
}

int main() {
    IntWrapper w(42);
    int x = w;
    double d = (double)w;
    int sum = w + 1;
    int sum_rev = 10 + w;
    int arg_result = add_ten(w);
    std::printf("x=%d d=%.1f sum=%d sum_rev=%d arg=%d\n", x, d, sum, sum_rev, arg_result);
    return 0;
}
