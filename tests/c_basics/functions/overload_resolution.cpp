#include <cstdio>

// Every call below has exactly one best match.  The compiler must pick it
// instead of reporting an ambiguity.

int pick_int(int value) {
    return 1;
}

int pick_long(long value) {
    return 2;
}

int pick_double(double value) {
    return 3;
}

int pick_short(short value) {
    return 4;
}

class Holder {
public:
    int value;
    Holder(int v) : value(v) {}
    int get() const { return value; }
    int get(int extra) const { return value + extra; }
};

int main() {
    int i = 3;
    long l = 4;
    double d = 5.5;
    float f = 6.5f;
    char c = 'x';
    Holder holder(9);

    std::printf("%d\n", pick_int(i));    // the exact match wins
    std::printf("%d\n", pick_long(l));   // the exact match wins
    std::printf("%d\n", pick_double(d)); // the exact match wins
    std::printf("%d\n", pick_double(f)); // the promotion wins over the conversion
    std::printf("%d\n", pick_int(c));    // the promotion wins over the conversion
    std::printf("%d\n", pick_short(c));  // the exact match wins
    std::printf("%d\n", holder.get());     // the overload without an argument
    std::printf("%d\n", holder.get(1));   // the overload with an argument
    return 0;
}
