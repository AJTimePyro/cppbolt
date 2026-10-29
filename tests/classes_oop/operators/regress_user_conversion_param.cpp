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

int take_int(int x) {
    return x + 10;
}

double take_double(double d) {
    return d + 2.0;
}

int return_converted(IntWrapper w) {
    return w;
}

int main() {
    IntWrapper w(42);
    IntWrapper w2 = 100;

    int r_int = take_int(w);
    double r_double = take_double((double)w);
    int r_ret = return_converted(w2);
    int r_bin1 = w + 5;
    int r_bin2 = 50 - w;
    int r_assign = w;

    std::printf("take_int=%d take_double=%.1f ret=%d bin1=%d bin2=%d assign=%d\n",
                r_int, r_double, r_ret, r_bin1, r_bin2, r_assign);
    return 0;
}
