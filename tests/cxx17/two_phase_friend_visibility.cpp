#include <cstdio>

namespace N {
struct Early { friend void hidden(Early) { std::printf("early\n"); } };
}

template <class T> void nondep(T) {
    N::Early e;
    hidden(e); // non-dependent: ADL is frozen at the definition
}
template <class T> void dep(T t) {
    hidden(t); // dependent: ADL also runs at the instantiation
}

namespace N {
struct Late { friend void hidden(Late) { std::printf("late\n"); } };
}

int main() {
    nondep(1);
    dep(N::Late{});
    return 0;
}
