#include <cstdio>

// The variant resolves the same overload sets with a different shape: the
// candidates live in a namespace and the methods are named differently.

namespace geometry {

int measure(int value) {
    return 1;
}

int measure(long value) {
    return 2;
}

int measure(double value) {
    return 3;
}

int measure(short value) {
    return 4;
}

} // namespace geometry

class Reading {
public:
    int amount;
    Reading(int v) : amount(v) {}
    int current() const { return amount; }
    int current(int offset) const { return amount + offset; }
};

int main() {
    int i = 3;
    long l = 4;
    double d = 5.5;
    float f = 6.5f;
    char c = 'x';
    Reading reading(9);

    std::printf("%d\n", geometry::measure(i));
    std::printf("%d\n", geometry::measure(l));
    std::printf("%d\n", geometry::measure(d));
    std::printf("%d\n", geometry::measure(f));
    std::printf("%d\n", geometry::measure(c));
    std::printf("%d\n", geometry::measure(c));
    std::printf("%d\n", reading.current());
    std::printf("%d\n", reading.current(1));
    return 0;
}
