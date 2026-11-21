#include <cstdio>

namespace N {
struct S {
    friend void hidden(S) { std::printf("hidden friend\n"); }
};
}

template <typename T> void call(T value) { hidden(value); }

int main() {
    call(N::S{});
    return 0;
}
