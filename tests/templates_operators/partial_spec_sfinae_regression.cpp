#include <cstdio>

struct Yes {
    using type = int;
    int get() const { return 7; }
};
struct No {};

// P4: a partial specialization whose dependent nested type is missing is
// discarded without blocking the primary template.
template <typename T, typename U> struct HasType { static const int v = 0; };
template <typename T> struct HasType<T, decltype(alignof(typename T::type))> {
    static const int v = 1;
};

// P3: expression SFINAE and trailing return types still work.
template <typename T> auto probe(T t) -> decltype(t.get()) { return t.get(); }
int probe(No) { return -1; }

int main() {
    Yes y;
    No n;
    std::printf("%d %d %d %d\n", HasType<Yes, size_t>::v,
                HasType<No, size_t>::v, probe(y), probe(n));
    return 0;
}
