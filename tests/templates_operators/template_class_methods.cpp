#include <cstdio>

template <typename T>
class Accumulator {
    T total;
public:
    Accumulator(T init);
    void add(T val);
    T get_total() const;
};

template <typename T>
Accumulator<T>::Accumulator(T init) : total(init) {}

template <typename T>
void Accumulator<T>::add(T val) {
    total += val;
}

template <typename T>
T Accumulator<T>::get_total() const {
    return total;
}

int main() {
    Accumulator<int> acc(10);
    acc.add(5);
    acc.add(7);
    std::printf("acc int: %d\n", acc.get_total());

    Accumulator<double> acc_d(1.5);
    acc_d.add(2.25);
    std::printf("acc double: %.2f\n", acc_d.get_total());
    return 0;
}
