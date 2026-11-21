#include <cstdio>

// Partial specializations whose parameters are deduced from at least one
// argument remain valid even when they also appear in non-deduced contexts.

template<class...> using void_t = void;
template<class A, class B> struct Pair {};
template<int A, class B> struct NPair {};
template<class...> struct Tuple {};

struct HasInt { using type = int; };

// T is deduced from the first argument; void_t's argument is non-deduced.
template<class T, class = void> struct A { static constexpr int value = 0; };
template<class T> struct A<T, void_t<typename T::type>> { static constexpr int value = 1; };

// T is deduced from the first argument; the alias expands to a dependent
// nested type in the second.
template<class T> using Only = typename T::type;
template<class T, class = void> struct B { static constexpr int value = 0; };
template<class T> struct B<T, Only<T>> { static constexpr int value = 2; };

// A non-type parameter and a type parameter, both deduced.
template<class X> struct C { static constexpr int value = 0; };
template<int N, class T> struct C<NPair<N, T>> { static constexpr int value = 3; };

// An alias whose expansion mixes a deducible part and a dependent nested part.
template<class T> using V = Pair<T, typename T::type>;
template<class X> struct D { static constexpr int value = 0; };
template<class T> struct D<V<T>> { static constexpr int value = 4; };

// A pack alias is deduced from the pack expansion.
template<class... Ts> using All = Tuple<Ts...>;
template<class X> struct E { static constexpr int value = 0; };
template<class... Ts> struct E<All<Ts...>> { static constexpr int value = 5; };

int main() {
    std::printf("A<HasInt>            = %d\n", A<HasInt>::value);
    std::printf("A<int>               = %d\n", A<int>::value);
    std::printf("B<HasInt,int>        = %d\n", B<HasInt, int>::value);
    std::printf("C<NPair<7,double>>   = %d\n", C<NPair<7, double>>::value);
    std::printf("D<Pair<HasInt,int>>  = %d\n", D<Pair<HasInt, int>>::value);
    std::printf("E<Tuple<int,double>> = %d\n", E<Tuple<int, double>>::value);
    return 0;
}
