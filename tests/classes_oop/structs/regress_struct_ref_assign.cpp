#include <cstdio>

struct Counter {
    int val;
    Counter() : val(0) {}
    Counter(int v) : val(v) {}
    Counter& operator=(const Counter& other) {
        val = other.val + 100;
        return *this;
    }
};

void assign_through_ref(Counter& target, const Counter& src) {
    target = src;
}

int main() {
    Counter c1(10);
    Counter c2(5);
    Counter& ref = c1;
    ref = c2;
    std::printf("c1.val: %d\n", c1.val);
    assign_through_ref(c1, c2);
    std::printf("c1.val after helper: %d\n", c1.val);
    return 0;
}
