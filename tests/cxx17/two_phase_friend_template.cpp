#include <cstdio>

namespace N {
template <class T> struct S {
    friend void hidden(S) { std::printf("friend\n"); }
};
}

int main() {
    hidden(N::S<int>{});
    return 0;
}
