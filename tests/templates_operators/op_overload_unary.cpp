#include <cstdio>

class Counter {
public:
    int count;
    Counter(int c = 0) : count(c) {}

    Counter& operator++() {
        ++count;
        return *this;
    }
    Counter operator-() const {
        return Counter(-count);
    }
    bool operator!() const {
        return count == 0;
    }
};

int main() {
    Counter c(0);
    std::printf("is zero: %d\n", !c ? 1 : 0);
    ++c;
    std::printf("after ++: %d\n", c.count);
    Counter neg = -c;
    std::printf("neg count: %d\n", neg.count);
    return 0;
}
