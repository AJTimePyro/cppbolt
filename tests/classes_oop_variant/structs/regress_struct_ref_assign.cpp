#include <cstdio>

struct Box {
    int weight;
    Box() : weight(0) {}
    Box(int w) : weight(w) {}
    Box& operator=(const Box& o) {
        weight = o.weight * 2;
        return *this;
    }
};

void do_assign(Box& b, const Box& val) {
    b = val;
}

int main() {
    Box b1(20);
    Box b2(7);
    Box& r = b1;
    r = b2;
    std::printf("b1: %d\n", b1.weight);
    do_assign(b1, b2);
    std::printf("b1 again: %d\n", b1.weight);
    return 0;
}
