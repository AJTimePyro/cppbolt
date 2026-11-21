#include <cstdio>

namespace N1 {
    int get() { return 1; }
    using Type = decltype(get());
}

namespace N2 {
    double get() { return 2.0; }
    using Type = decltype(get());
}

int main() {
    N1::Type a = 1;
    N2::Type b = 2.0;
    std::printf("%zu %zu\n", sizeof(a), sizeof(b));
    return 0;
}
