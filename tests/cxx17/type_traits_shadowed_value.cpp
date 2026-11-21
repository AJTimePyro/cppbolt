#include <cstdio>
#include <type_traits>

template <bool B> struct Flag { static const int value = B ? 1 : 0; };

// A derived trait shadows its base's `value`; the derived one must win both
// in a constant expression and at runtime.
template <typename T> struct BaseValue { static const bool value = false; };
template <typename T> struct DerivedValue : BaseValue<T> {
    static const bool value = true;
};

// The same shadowing feeds a substitution decision.
template <typename T>
std::enable_if_t<DerivedValue<T>::value, int> pick(T) { return 1; }
template <typename T>
std::enable_if_t<!DerivedValue<T>::value, int> pick(T) { return 0; }

int main() {
    std::printf("%d %d %d %d\n",
        (int)BaseValue<int>::value,
        (int)DerivedValue<int>::value,
        Flag<DerivedValue<int>::value>::value,
        pick(1));
    return 0;
}
