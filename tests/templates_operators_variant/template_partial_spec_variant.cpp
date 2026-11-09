#include <iostream>

template <typename T, typename U>
struct PairKind {
    static void report() { std::cout << "distinct types" << std::endl; }
};

template <typename T>
struct PairKind<T, T> {
    static void report() { std::cout << "identical types" << std::endl; }
};

template <typename T, typename U>
struct PairKind<T*, U*> {
    static void report() { std::cout << "both pointers" << std::endl; }
};

int main() {
    PairKind<int, double>::report();
    PairKind<int, int>::report();
    PairKind<int*, double*>::report();
    return 0;
}
