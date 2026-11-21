#include <cstdio>

// A decltype operand that cannot be rendered as re-parsable source must be a
// hard error during confirmation, not a silent fallback to the primary.
template<class T> struct Box { using type = T; };

template<class T, class U>
struct X { static constexpr int value = 0; };
template<class T>
struct X<T, typename Box<decltype([]() { return T(); })>::type> {
    static constexpr int value = 1;
};

int main() {
    std::printf("%d\n", X<int, int>::value);
    return 0;
}
