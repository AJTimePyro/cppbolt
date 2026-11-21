#include <cstdio>

struct Base { typedef int value_type; };
struct DerivedOK : Base {};
struct DirectType { typedef int value_type; };
struct B1 { typedef int value_type; };
struct B2 { typedef char other; };
struct Mid : B1 { double value_type; };
struct Leaf : B2, Mid {};

template<class T> int probe(typename T::value_type*) { return 1; }

int main() {
    printf("%d %d\n", probe<DerivedOK>(0), probe<DirectType>(0));
    return 0;
}
