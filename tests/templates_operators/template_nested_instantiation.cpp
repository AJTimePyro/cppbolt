#include <cstdio>

template <typename T> struct Wrap { T value; Wrap(T v) : value(v) {} };
template <typename T> struct Pair {
    Wrap<T> first;
    Wrap<T> second;
    Pair(T a, T b) : first(a), second(b) {}
};

// Repeated instantiation of Wrap<int> must reuse the completed cache entry.
int main() {
    Pair<int> p(3, 7);
    Pair<int> q(1, 2);
    std::printf("%d %d\n", p.first.value + q.second.value, (int)sizeof(Wrap<int>));
    return 0;
}
