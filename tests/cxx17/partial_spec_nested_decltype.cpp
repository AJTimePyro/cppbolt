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

// Other operand forms. Each is rendered as source when the qualified pattern
// is stringified, then reparsed during confirmation. A non-matching second
// argument makes sure none of them over-match.
template<class T, class U>
struct CS { static constexpr int value = 0; };
template<class T>
struct CS<T, typename Box<decltype(static_cast<T>(0))>::type> {
    static constexpr int value = 1;
};

template<class T, class U>
struct TS { static constexpr int value = 0; };
template<class T>
struct TS<T, typename Box<decltype(true ? T() : T())>::type> {
    static constexpr int value = 1;
};

template<class T, class U>
struct SS { static constexpr int value = 0; };
template<class T>
struct SS<T, typename Box<decltype(T() << 1)>::type> {
    static constexpr int value = 1;
};

template<class T, class U>
struct RS { static constexpr int value = 0; };
template<class T>
struct RS<T, typename Box<decltype(+((T*)0)[0])>::type> {
    static constexpr int value = 1;
};

// A value-returning subscript keeps declval<T>()[0] a prvalue, so the result
// agrees with native C++ (a pointer subscript would yield a reference, which
// cppbolt's decltype does not model).
struct ArrayLike { int operator[](int) const { return 0; } };
template<class T, class U>
struct DS { static constexpr int value = 0; };
template<class T>
struct DS<T, typename Box<decltype(std::declval<T>()[0])>::type> {
    static constexpr int value = 1;
};

template<class T, class U>
struct VS { static constexpr int value = 0; };
template<class T>
struct VS<T, typename Box<decltype(std::declval<T>() << 1)>::type> {
    static constexpr int value = 1;
};

int main() {
    std::printf("H<int,int>=%d H<int,double>=%d\n", H<int,int>::value, H<int,double>::value);
    std::printf("G<HasGet,int>=%d G<NoGet,int>=%d\n", G<HasGet,int>::value, G<NoGet,int>::value);
    std::printf("L<int,int>=%d L<int,double>=%d\n", L<int,int>::value, L<int,double>::value);
    std::printf("M<int,int>=%d N<WithGet*,int>=%d\n", M<int,int>::value, N<WithGet*,int>::value);
    std::printf("CS<int,int>=%d CS<int,double>=%d\n", CS<int,int>::value, CS<int,double>::value);
    std::printf("TS<int,int>=%d TS<int,double>=%d\n", TS<int,int>::value, TS<int,double>::value);
    std::printf("SS<int,int>=%d SS<int,double>=%d\n", SS<int,int>::value, SS<int,double>::value);
    std::printf("RS<int,int>=%d RS<int,double>=%d\n", RS<int,int>::value, RS<int,double>::value);
    std::printf("DS<ArrayLike,int>=%d DS<ArrayLike,double>=%d\n",
                DS<ArrayLike,int>::value, DS<ArrayLike,double>::value);
    std::printf("VS<int,int>=%d VS<int,double>=%d\n", VS<int,int>::value, VS<int,double>::value);
    return 0;
}
