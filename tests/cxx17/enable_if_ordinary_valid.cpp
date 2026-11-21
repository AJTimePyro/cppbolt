#include <cstdio>
#include <type_traits>

// A valid enable_if member access in an ordinary context stays valid, and
// SFINAE detection idioms continue to select the right overloads.

using Good = std::enable_if<true, int>::type;

template<class T> std::enable_if_t<std::is_integral<T>::value, int> pick(T) { return 1; }
template<class T> std::enable_if_t<!std::is_integral<T>::value, int> pick(T) { return 2; }

template<class T, class = void> struct has_nested : std::false_type {};
template<class T> struct has_nested<T, std::void_t<typename T::type>> : std::true_type {};
struct WithType { using type = int; };

template<class T, class = void> struct Sel { static constexpr int value = 0; };
template<class T> struct Sel<T, std::enable_if_t<std::is_integral<T>::value>> { static constexpr int value = 1; };

int main() {
    Good g = 5;
    std::printf("pick=%d%d has=%d%d sel=%d%d g=%d\n",
        pick(1), pick(1.5),
        (int)has_nested<WithType>::value, (int)has_nested<int>::value,
        Sel<int>::value, Sel<double>::value, g);
    return 0;
}
