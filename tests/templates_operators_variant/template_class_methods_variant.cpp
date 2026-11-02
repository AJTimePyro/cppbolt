#include <cstdio>

template <typename T, typename U>
class PairAccumulator {
    T total_t;
    U total_u;
public:
    PairAccumulator(T first, U second);
    ~PairAccumulator();
    void accumulate(T a, U b);
    T get_first() const;
    U get_second() const;
};

template <typename T, typename U>
PairAccumulator<T, U>::PairAccumulator(T first, U second) : total_t(first), total_u(second) {
    std::printf("created pair\n");
}

template <typename T, typename U>
PairAccumulator<T, U>::~PairAccumulator() {
    std::printf("destroyed pair\n");
}

template <typename T, typename U>
void PairAccumulator<T, U>::accumulate(T a, U b) {
    total_t += a;
    total_u += b;
}

template <typename T, typename U>
T PairAccumulator<T, U>::get_first() const {
    return total_t;
}

template <typename T, typename U>
U PairAccumulator<T, U>::get_second() const {
    return total_u;
}

int main() {
    {
        PairAccumulator<int, double> p(10, 2.5);
        p.accumulate(5, 1.25);
        p.accumulate(20, 3.75);
        std::printf("result: %d, %.2f\n", p.get_first(), p.get_second());
    }
    return 0;
}
