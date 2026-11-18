#include <cstdio>

// A function parameter pack before the deduced pack is a non-deduced context:
// the trailing pack takes all three arguments and the leading pack stays empty.
template <typename... Ts, typename... Us>
int total(Ts... ts, Us... us) {
    return sizeof...(Ts) * 10 + sizeof...(Us);
}

int main() {
    std::printf("%d\n", total(1, 2, 'a'));
    return 0;
}
