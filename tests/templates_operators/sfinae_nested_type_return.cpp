#include <cstdio>

struct IntBox { using type = int; };

// A dependent nested type is also valid in a function template's return type.
template <typename T> typename T::type make(int v) { return v; }

int main() {
    std::printf("%d\n", make<IntBox>(7));
    return 0;
}
