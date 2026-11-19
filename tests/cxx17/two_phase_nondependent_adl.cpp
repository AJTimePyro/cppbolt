#include <cstdio>

namespace N {
struct S {};
void pick(const S&) { std::printf("definition\n"); }
}

template <class T> void caller(T) {
    N::S s;
    pick(s); // non-dependent: ADL is frozen at the template definition
}

namespace N {
void pick(S&) { std::printf("later\n"); }
}

int main() {
    caller(1);
    return 0;
}
