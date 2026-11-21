#include <cstdio>

namespace N {
    struct S {};
}

template <typename T>
int run(T t) {
    auto local = t;
    return tag(local);
}

namespace N {
    int tag(S) { return 42; }
}

int main() {
    N::S s;
    std::printf("%d\n", run(s));
    return 0;
}
