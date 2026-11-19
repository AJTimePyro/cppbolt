#include <cstdio>

template <class... Ts, class U> int cnt(U) { return sizeof...(Ts); }
template <class T, class... Ts> int tail(T, Ts...) { return sizeof...(Ts); }

int empty_first() { return cnt(3); }
int int_first() { return cnt<int>(3); }

int main() {
    // Empty pack followed by an explicit single-element int pack.
    std::printf("%d %d\n", cnt(3), cnt<int>(3));
    // Reverse order: the cached specialization must not shadow re-deduction.
    std::printf("%d %d\n", cnt<int>(3), cnt(3));
    // Repeated calls to both specializations.
    std::printf("%d %d %d %d\n", cnt(3), cnt<int>(3), cnt<int>(3), cnt(3));
    // The same specializations requested from different functions.
    std::printf("%d %d\n", empty_first(), int_first());
    // Different pack element types and a multi-element pack.
    std::printf("%d %d %d\n", cnt<char>(3), cnt<double>(3), cnt<int, char>(3));
    // Existing trailing-pack behavior is unchanged.
    std::printf("%d %d\n", tail(1), tail(1, 2.0, 'x'));
    return 0;
}
