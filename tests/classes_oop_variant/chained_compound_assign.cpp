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

int main() {
    RunningTotal base(40);
    RunningTotal first(9);
    RunningTotal second(5);
    (base += first) += second;
    std::printf("%d\n", base.amount);
    return 0;
}
