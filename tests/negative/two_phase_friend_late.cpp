#include <cstdio>
namespace N { struct Early {}; }

template <class T> void nondep(T) {
    N::Early e;
    hidden(e); // non-dependent: must not find a friend declared later
}

namespace N {
struct Other { friend void hidden(Early) { std::printf("late\n"); } };
}

int main() { nondep(1); }
