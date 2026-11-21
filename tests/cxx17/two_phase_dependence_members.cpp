#include <cstdio>

namespace N {
struct S {};
void pick(S) { std::printf("N::pick\n"); }
}

// Class-template method with a dependent call.
template <class T> struct Holder {
    void call(T value) { pick(value); }
};

// Member template whose signature and body both contain dependent calls.
struct Dispatch {
    template <class T> void call(T value) { pick(value); }
};

int main() {
    Holder<N::S> h;
    h.call(N::S{});
    Dispatch d;
    d.call(N::S{});
    return 0;
}
