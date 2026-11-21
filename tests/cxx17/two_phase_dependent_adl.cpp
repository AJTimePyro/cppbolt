#include <cstdio>

namespace N {
struct S {};
void pick(const S&) { std::printf("definition\n"); }
}

template <class T> void caller(T value) {
    pick(value); // dependent: ADL runs at instantiation
}

namespace N {
void pick(S&) { std::printf("later\n"); }
}

int main() {
    caller(N::S{});
    return 0;
}
