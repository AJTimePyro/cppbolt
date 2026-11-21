#include <cstdio>
#include <type_traits>

struct S { static constexpr int value = 1; };

// Qualified static constexpr member.
static_assert(S::value == 1, "qualified member");

// Variable-template values.
static_assert(std::is_integral_v<int>, "integral");
static_assert(!std::is_integral_v<double>, "not integral");

// Local constexpr variable initialized from a qualified member.
constexpr int local = S::value;
static_assert(local == 1, "local constexpr");

// Static member of a class template.
template<class T> struct Trait { static constexpr int value = 2; };
static_assert(Trait<int>::value == 2, "trait member");

// A dependent assertion inside a template must not be rejected before
// substitution; it is checked, if at all, once T is known.
template<class T> void check() {
    static_assert(sizeof(T) > 0, "positive size");
}

int main() {
    check<int>();
    std::printf("ok\n");
    return 0;
}
