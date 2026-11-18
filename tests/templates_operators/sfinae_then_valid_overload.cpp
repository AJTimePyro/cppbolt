#include <cstdio>

struct Bad {};
struct Good { using type = int; };

// The first candidate is discarded for Bad (no Bad::type); the second call
// must then form a valid candidate for the different argument type.
template <typename T> int f(typename T::type* p) { return 1; }
template <typename T> int f(long) { return 2; }

int main() {
    int v = 0;
    std::printf("%d %d\n", f<Bad>(0L), f<Good>(&v));
    return 0;
}
