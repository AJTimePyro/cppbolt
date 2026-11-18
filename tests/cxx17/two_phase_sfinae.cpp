#include <cstdio>
#include <type_traits>
#include <utility>

namespace N {
struct Yes {};
int value(Yes) { return 42; }
}
struct No {};

// Dependent call in a trailing return type.
template <typename T> auto trail(T t) -> decltype(value(t)) { return value(t); }

// void_t detection and enable_if dispatch over a dependent ADL call.
template <typename T, typename = void> struct has_value : std::false_type {};
template <typename T>
struct has_value<T, std::void_t<decltype(value(std::declval<T>()))> > : std::true_type {};

template <typename T>
std::enable_if_t<has_value<T>::value, int> pick(T) { return value(T{}); }
template <typename T>
std::enable_if_t<!has_value<T>::value, int> pick(T) { return 0; }

int main() {
    std::printf("%d %d %d %d %d\n",
        trail(N::Yes{}),
        pick(N::Yes{}), pick(No{}),
        (int)has_value<N::Yes>::value, (int)has_value<No>::value);
    return 0;
}
