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

template <typename... A>
int fold(A... a) {
    return (... + a);
}

int main() {
    int f = fold(Num(10), Num(5), Num(2));
    int c = (Num(10) + Num(5)) + Num(2);
    std::printf("fold=%d chain=%d calls=%d\n", f, c, calls);
    return 0;
}
