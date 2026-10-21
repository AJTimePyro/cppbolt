#include <cstdio>

class Gauge {
    int reading;
public:
    Gauge(int start) : reading(start) {}
    int value() { return reading; }
    int value() const { return reading + 2; }
};

int main() {
    Gauge gauge(35);
    const Gauge fixed_gauge(35);
    std::printf("%d %d\n", gauge.value(), fixed_gauge.value());
    return 0;
}
