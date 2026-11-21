#include <cstdio>

struct OnlyType { using type = int; };
struct OnlyOther { using other = int; };

// Each argument discards the overload whose dependent nested type is missing.
template <typename T> int rank(T, typename T::type*) { return 1; }
template <typename T> int rank(T, typename T::other*) { return 2; }

int main() {
    int i = 0;
    std::printf("%d %d\n", rank(OnlyType{}, &i), rank(OnlyOther{}, &i));
    return 0;
}
