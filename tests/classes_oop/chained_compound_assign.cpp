#include <cstdio>

class Accumulator {
public:
    int total;
    Accumulator(int value) : total(value) {}
    Accumulator& operator+=(const Accumulator& other) {
        total += other.total;
        return *this;
    }
};

int main() {
    Accumulator a(18);
    Accumulator b(7);
    Accumulator c(3);
    (a += b) += c;
    std::printf("%d\n", a.total);
    return 0;
}
