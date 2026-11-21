#include <cstdio>

template <class B>
struct Base { typedef int type; };

template <class T>
struct Wrapper : Base<T> {};

template <class T>
struct Holder {
    char data[sizeof(typename Wrapper<T>::type)];
};

// Wrapper<int> has not been instantiated at this point.
Holder<int> first;

// Instantiate Wrapper<int> explicitly, then use it again.
Wrapper<int> explicit_earlier;
Holder<int> second;

int main() {
    printf("%zu %zu\n", sizeof(first.data), sizeof(second.data));
    return 0;
}
