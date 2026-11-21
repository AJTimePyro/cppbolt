#include <cstdio>

// An alias whose expansion mixes a deducible component and a dependent nested
// type: deduction uses the structural part, confirmation validates the nested
// type after the parameters are bound.

template <class A, class B> struct Pair {};
template <class A, class B, class C> struct Triple {};

struct HasInt { using type = int; };
struct HasDouble { using type = double; };
struct NoType {};

// Reported case: dependent nested type in a later alias argument.
template <class T> using V = Pair<T, typename T::type>;
template <class T> struct S { static constexpr int value = 0; };
template <class T> struct S<V<T>> { static constexpr int value = 1; };

// Dependent nested type in the first alias argument, before T is deduced.
template <class T> using First = Pair<typename T::type, T>;
template <class T> struct Q { static constexpr int value = 0; };
template <class T> struct Q<First<T>> { static constexpr int value = 2; };

// Two independently deducible parameters plus a dependent nested type.
template <class T, class U> using V2 = Triple<T, U, typename T::type>;
template <class X> struct M { static constexpr int value = 0; };
template <class T, class U> struct M<V2<T, U>> { static constexpr int value = 3; };

// The alias expands entirely to a dependent nested type, so T is deduced from
// another argument and confirmation still validates the nested type.
template <class T> using Only = typename T::type;
template <class T, class = void> struct OT { static constexpr int value = 0; };
template <class T> struct OT<T, Only<T>> { static constexpr int value = 4; };

// The alias nested inside another class-template pattern. Confirmation must
// compare the instantiated alias result with the recovered inner argument.
template <class X> struct Wrap {};
template <class X> struct Nested { static constexpr int value = 0; };
template <class T> struct Nested<Wrap<V<T>>> { static constexpr int value = 5; };

int main() {
    std::printf("S<Pair<HasInt,int>>    = %d\n", S<Pair<HasInt, int>>::value);
    std::printf("S<Pair<HasInt,double>> = %d\n", S<Pair<HasInt, double>>::value);
    std::printf("S<Pair<NoType,int>>    = %d\n", S<Pair<NoType, int>>::value);
    std::printf("Q<Pair<int,HasInt>>    = %d\n", Q<Pair<int, HasInt>>::value);
    std::printf("Q<Pair<int,NoType>>    = %d\n", Q<Pair<int, NoType>>::value);
    std::printf("M<Triple<HasInt,double,int>> = %d\n", M<Triple<HasInt, double, int>>::value);
    std::printf("M<Triple<HasInt,double,double>> = %d\n", M<Triple<HasInt, double, double>>::value);
    std::printf("OT<HasInt,int>         = %d\n", OT<HasInt, int>::value);
    std::printf("OT<HasInt,double>      = %d\n", OT<HasInt, double>::value);
    std::printf("OT<NoType,int>         = %d\n", OT<NoType, int>::value);
    std::printf("Nested<Wrap<Pair<HasInt,int>>> = %d\n", Nested<Wrap<Pair<HasInt, int>>>::value);
    return 0;
}
