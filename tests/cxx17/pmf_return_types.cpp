#include <cstdio>

struct R { int a; int b; };

struct S {
    int field;
    int i(int n) { return n + 1; }
    double d(double x) { return x + 0.5; }
    long long ll(long long v) { return v + 3; }
    R rec(int n) { return R{n, n + 1}; }
    int* ptr(int* p) { return p; }
    int& ref() { return field; }
    void nothing(int n) { (void)n; }
};

int main() {
    S s{};
    int (S::* pi)(int) = &S::i;
    double (S::* pd)(double) = &S::d;
    long long (S::* pl)(long long) = &S::ll;
    R (S::* pr)(int) = &S::rec;
    int* (S::* pp)(int*) = &S::ptr;
    void (S::* pv)(int) = &S::nothing;
    int& (S::* pref)() = &S::ref;

    int iv = (s.*pi)(1);
    double dv = (s.*pd)(2.0);
    long long lv = (s.*pl)(5LL);
    R rv = (s.*pr)(7);
    int value = 9;
    int* pv2 = (s.*pp)(&value);
    (s.*pv)(0);
    (s.*pref)() = 42;

    std::printf("%d %.1f %lld %d %d %d %d\n", iv, dv, lv, rv.a, rv.b, *pv2, s.field);
    return 0;
}
