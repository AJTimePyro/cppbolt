#include <initializer_list>
#include <iostream>

template <class T>
struct Box {
    T value;
};

struct Wrap {
    int total;
    Wrap(Box<int> b) : total(b.value) {}
};

template <class T>
struct Nested {
    T value;
};

struct Deep {
    int total;
    Deep(Nested<Box<int>> b) : total(b.value.value) {}
};

int main() {
    Box<int> b;
    b.value = 21;
    Wrap w(b);
    Nested<Box<int>> n;
    n.value = b;
    Deep d(n);
    std::cout << "wrap: " << w.total << std::endl;
    std::cout << "deep: " << d.total << std::endl;
    return 0;
}