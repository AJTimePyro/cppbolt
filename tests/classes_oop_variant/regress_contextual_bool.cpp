#include <cstdio>

struct Checker {
    int state;
    Checker(int s) : state(s) {}
    explicit operator bool() const {
        return state != 0;
    }
};

int main() {
    Checker c1(42);
    Checker c0(0);

    if (c1) {
        std::printf("c1 truthy\n");
    }
    if (!c0) {
        std::printf("!c0 falsy negated\n");
    }
    std::printf("ternary c0: %d\n", c0 ? 10 : 20);
    int iter = 0;
    for (; c1 && iter < 1; iter++) {
        std::printf("for loop iter %d\n", iter);
    }
    return 0;
}
