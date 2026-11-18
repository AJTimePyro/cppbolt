#include <cstdio>

int calls = 0;

struct Num {
    int v;
    Num(int x) : v(x) {}
};

int operator+(const Num& a, const Num& b) {
    ++calls;
    return a.v + b.v;
}

int main() {
    int chain = Num(1) + Num(2) + Num(3) + Num(4) + Num(5)
              + Num(6) + Num(7) + Num(8) + Num(9) + Num(10)
              + Num(11) + Num(12) + Num(13) + Num(14) + Num(15)
              + Num(16) + Num(17) + Num(18) + Num(19) + Num(20);
    std::printf("chain=%d calls=%d\n", chain, calls);
    return 0;
}
