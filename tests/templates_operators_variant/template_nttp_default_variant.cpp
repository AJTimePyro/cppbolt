#include <iostream>

template <int Base = 10, int Multiplier = 2>
struct ScalarCalculator {
    int compute(int x) const {
        return Base + x * Multiplier;
    }
};

int main() {
    ScalarCalculator<> calc1;
    ScalarCalculator<5> calc2;
    ScalarCalculator<5, 3> calc3;
    std::cout << calc1.compute(3) << " " << calc2.compute(3) << " " << calc3.compute(3) << std::endl;
    return 0;
}
