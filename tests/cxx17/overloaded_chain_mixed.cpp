#include <cstdio>

int calls = 0;

struct R {
    int v;
    R(int x) : v(x) {}
};

struct S {
    int v;
    S(int x) : v(x) {}
    S(const R& r) : v(r.v) {}
};

R operator+(const R& a, const R& b) {
    ++calls;
    return R(a.v + b.v);
}

S operator+(const S& a, const S& b) {
    ++calls;
    return S(a.v + b.v);
}

int main() {
    // The middle step converts R to S, so the compiler recompiles the prefix
    // to apply the conversion.
    S r = R(1) + R(2) + S(30) + S(40);
    std::printf("r=%d calls=%d\n", r.v, calls);
    return 0;
}
