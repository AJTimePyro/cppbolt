#include <cstdio>
#include <utility>

// A decltype nested inside a qualified dependent type is a non-deduced
// context: other pattern arguments bind the parameters, then the fully
// substituted qualified type is resolved and compared.

template<class T> struct Box { using type = T; };

template<class T, class U>
struct H { static constexpr int value = 0; };
template<class T>
struct H<T, typename Box<decltype(T())>::type> { static constexpr int value = 1; };

// An invalid decltype operand rejects the specialization; the primary wins.
struct HasGet { int get() const { return 7; } };
struct NoGet {};
template<class T, class U>
struct G { static constexpr int value = 0; };
template<class T>
struct G<T, typename Box<decltype(std::declval<T>().get())>::type> {
    static constexpr int value = 2;
};

// Other expression forms inside the nested decltype: binary operators and
// arrow member access. Their spelling must not be confused with template
// angle brackets when the qualified pattern name is parsed.
template<class T, class U>
struct M { static constexpr int value = 0; };
template<class T>
struct M<T, typename Box<decltype(T() + T())>::type> { static constexpr int value = 5; };

struct WithGet { int get() const { return 7; } };
template<class T, class U>
struct N { static constexpr int value = 0; };
template<class T>
struct N<T, typename Box<decltype(T()->get())>::type> { static constexpr int value = 6; };

// Interaction with alias patterns in the first argument.
template<class T> using Id = T;
template<class T, class U>
struct L { static constexpr int value = 0; };
template<class T>
struct L<Id<T>, typename Box<decltype(T())>::type> { static constexpr int value = 4; };

int main() {
    std::printf("H<int,int>=%d H<int,double>=%d\n", H<int,int>::value, H<int,double>::value);
    std::printf("G<HasGet,int>=%d G<NoGet,int>=%d\n", G<HasGet,int>::value, G<NoGet,int>::value);
    std::printf("L<int,int>=%d L<int,double>=%d\n", L<int,int>::value, L<int,double>::value);
    std::printf("M<int,int>=%d N<WithGet*,int>=%d\n", M<int,int>::value, N<WithGet*,int>::value);
    return 0;
}
