#include <cstdio>

struct S { int x; int f() { return x; } };
int gv = 9;

template <int S::* PM> int read(S& s) { return s.*PM; }
template <auto PM> int read_auto(S& s) { return s.*PM; }
template <auto PM> int call_auto(S& s) { return (s.*PM)(); }
template <int (S::*PF)()> int call(S& s) { return (s.*PF)(); }
template <int* P> int read_global() { return *P; }

int main() {
    S s{8};
    std::printf("%d %d %d %d %d\n", read<&S::x>(s), read_auto<&S::x>(s),
                call_auto<&S::f>(s), call<&S::f>(s), read_global<&gv>());
    return 0;
}
