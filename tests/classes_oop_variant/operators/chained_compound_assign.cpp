#include <cstdio>

class RunningTotal {
public:
    int amount;
    RunningTotal(int start) : amount(start) {}
    RunningTotal& operator+=(const RunningTotal& rhs) {
        amount += rhs.amount;
        return *this;
    }
};

int chain(RunningTotal& base, RunningTotal& first, RunningTotal& second) {
    (base += first) += second;
    return base.amount;
}

int main() {
    RunningTotal base(40);
    RunningTotal first(9);
    RunningTotal second(5);
    std::printf("%d\n", chain(base, first, second));
    return 0;
}
