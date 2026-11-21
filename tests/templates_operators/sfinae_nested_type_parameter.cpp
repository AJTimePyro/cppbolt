#include <cstdio>

struct IntBox { using type = int; };
struct CharBox { using type = char; };

// The dependent nested type resolves to the member alias of each instantiated box.
template <typename T> int probe(typename T::type v) { return sizeof(v); }

int main() {
    std::printf("%d %d\n", probe<IntBox>(1), probe<CharBox>(1));
    return 0;
}
