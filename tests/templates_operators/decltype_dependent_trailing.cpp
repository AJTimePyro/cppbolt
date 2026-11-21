#include <cstdio>

// The trailing return type is substituted with the concrete argument before
// its decltype operand is analyzed.
template <typename T> struct Box { T value; };
template <typename T> auto unwrap(Box<T> b) -> decltype(b.value) { return b.value; }

struct HasMember { int m; };
struct NoMember {};
template <typename T> auto probe(Box<T> b) -> decltype(b.value.m) { return 1; }
int probe(Box<NoMember>) { return 2; }

int main() {
    Box<int> i; i.value = 8;
    Box<HasMember> h; h.value.m = 5;
    Box<NoMember> n;
    std::printf("%d %d %d\n", unwrap(i), probe(h), probe(n));
    return 0;
}
