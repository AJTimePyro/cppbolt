#include <cstdio>

struct S { int x; int f() { return x; } };

template <int S::* PM> bool data_is_null(S& s) { return PM == nullptr; }
template <int (S::* PF)()> bool fn_is_null() { return PF == nullptr; }

int main() {
    S s{7};
    std::printf("%d %d\n", data_is_null<nullptr>(s), fn_is_null<nullptr>());
    return 0;
}
