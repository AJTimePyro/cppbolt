#include <cstdio>

namespace N {
template <class T> struct S {
    friend int pick(S, T) { return sizeof(T); }
};
}

int main() {
    std::printf("%d %d %d\n",
        pick(N::S<int>{}, 0),
        pick(N::S<char>{}, 'a'),
        pick(N::S<int>{}, 0));
    return 0;
}
