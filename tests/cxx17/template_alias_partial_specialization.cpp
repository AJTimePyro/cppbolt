#include <cstdio>

// Alias templates expand before deduction, so their arguments are deducible
// in a class-template partial specialization (S<Ptr<T>> matches S<int*>).

template <class T> using Ptr = T*;
template <class T> struct A { static constexpr int value = 0; };
template <class T> struct A<Ptr<T>> { static constexpr int value = 1; };

// Nested type structure: a pointer-pointer alias and an alias to a class
// template instance both keep the pattern deducible.
template <class T> using PPtr = T**;
template <class T> struct B { static constexpr int value = 0; };
template <class T> struct B<PPtr<T>> { static constexpr int value = 2; };

template <class T> struct Wrap {};
template <class T> using Wrapper = Wrap<T>;
template <class T> struct C { static constexpr int value = 0; };
template <class T> struct C<Wrapper<T>> { static constexpr int value = 3; };

// Multiple template parameters in the alias pattern.
template <class X, class Y> struct Pair {};
template <class X, class Y> using AliasPair = Pair<X, Y>;
template <class T> struct D { static constexpr int value = 0; };
template <class X, class Y> struct D<AliasPair<X, Y>> { static constexpr int value = 4; };

// A default argument on the alias is filled before matching.
template <class X, class Y = int> struct Duo {};
template <class X, class Y = int> using AliasDuo = Duo<X, Y>;
template <class T> struct E { static constexpr int value = 0; };
template <class X, class Y> struct E<AliasDuo<X, Y>> { static constexpr int value = 5; };

// A pack alias expands to a pack pattern that deduces its elements.
template <class...> struct Tuple {};
template <class... Ts> using All = Tuple<Ts...>;
template <class T> struct F { static constexpr int value = 0; };
template <class... Ts> struct F<All<Ts...>> { static constexpr int value = 6; };

// Non-matching concrete arguments must still select the primary template:
// the alias pattern is checked structurally and not treated as a wildcard.
template <class T, class U> struct G { static constexpr int value = 0; };
template <class T, class U> struct G<T, Ptr<U>> { static constexpr int value = 7; };

// Partial ordering ranks an alias by its expansion: Ptr<T> is T*, so the more
// specialized const T* must win for a const pointer argument.
template <class T> struct H { static constexpr int value = 0; };
template <class T> struct H<Ptr<T>> { static constexpr int value = 8; };
template <class T> struct H<const T*> { static constexpr int value = 9; };

template <class T> using ConstPtr = const T*;
template <class T> struct I { static constexpr int value = 0; };
template <class T> struct I<Ptr<T>> { static constexpr int value = 10; };
template <class T> struct I<ConstPtr<T>> { static constexpr int value = 11; };

int main() {
    std::printf("A<int*>   = %d\n", A<int*>::value);
    std::printf("A<int>    = %d\n", A<int>::value);
    std::printf("A<int**>  = %d\n", A<int**>::value);
    std::printf("B<int**>  = %d\n", B<int**>::value);
    std::printf("B<int*>   = %d\n", B<int*>::value);
    std::printf("C<Wrap<int>> = %d\n", C<Wrap<int>>::value);
    std::printf("C<int>    = %d\n", C<int>::value);
    std::printf("D<Pair<int,double>> = %d\n", D<Pair<int,double>>::value);
    std::printf("D<int>    = %d\n", D<int>::value);
    std::printf("E<Duo<int>> = %d\n", E<Duo<int>>::value);
    std::printf("E<Duo<int,double>> = %d\n", E<Duo<int,double>>::value);
    std::printf("F<Tuple<int,double>> = %d\n", F<Tuple<int,double>>::value);
    std::printf("F<Tuple<>> = %d\n", F<Tuple<>>::value);
    std::printf("F<int>    = %d\n", F<int>::value);
    std::printf("G<int,int*> = %d\n", G<int, int*>::value);
    std::printf("G<int,double> = %d\n", G<int, double>::value);
    std::printf("H<const int*> = %d\n", H<const int*>::value);
    std::printf("H<int*>      = %d\n", H<int*>::value);
    std::printf("I<const int*> = %d\n", I<const int*>::value);
    return 0;
}
