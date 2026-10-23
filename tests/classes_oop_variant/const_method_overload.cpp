#include <cstdio>

class Gauge {
    int reading;
public:
    Gauge(int start) : reading(start) {}
    int value() { return reading; }
    int value() const { return reading + 2; }
};

int read_const(const Gauge& gauge) { return gauge.value(); }

int main() {
    Gauge gauge(35);
    const Gauge fixed_gauge(35);
    std::printf("%d %d\n", gauge.value(), read_const(fixed_gauge));
    return 0;
}
