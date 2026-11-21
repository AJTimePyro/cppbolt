#include <cstdio>

// A pointer-to-member class parameter is deducible, so the declaration is
// valid. Another specialization is selected to keep the program observable.
template<class X> struct S { static constexpr int value = 0; };
template<class T> struct S<int T::*> { static constexpr int value = 1; };
template<class T> struct S<T*> { static constexpr int value = 2; };

struct A { int x; };

int main() {
    std::printf("S<int*> = %d\n", S<int*>::value);
    return 0;
}
