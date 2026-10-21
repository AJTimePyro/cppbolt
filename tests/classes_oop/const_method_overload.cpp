#include <cstdio>

class Sensor {
    int reading;
public:
    Sensor(int value) : reading(value) {}
    int value() { return reading; }
    int value() const { return reading + 1; }
};

int main() {
    Sensor sensor(20);
    const Sensor constant_sensor(20);
    std::printf("%d %d\n", sensor.value(), constant_sensor.value());
    return 0;
}
