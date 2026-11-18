#include <cstdio>

struct Good {};
struct Bad {};

template <typename T> struct Traits { using type = T; };
template <> struct Traits<Bad> {};

// Traits<T>::type and an alias of that alias both resolve through aliases
// registered when the class template is instantiated.
template <typename T> int tag(T, typename Traits<T>::type* = nullptr) { return 1; }
int tag(Bad) { return 2; }

struct AliasBox { using type = Traits<Good>::type; };
template <typename T> int aliasTag(T, typename T::type* = nullptr) { return 3; }
int aliasTag(Bad) { return 4; }

int main() {
    std::printf("%d %d %d\n", tag(Good{}), tag(Bad{}), aliasTag(AliasBox{}));
    return 0;
}
