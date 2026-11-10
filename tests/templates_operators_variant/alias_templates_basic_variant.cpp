#include <iostream>

template <typename T>
struct Box {
    T val;
    Box(T v) : val(v) {}
    T get() const { return val; }
};

template <typename T>
using BoxPtr = Box<T*>;

template <typename T>
using DoubleBox = Box<Box<T>>;

int main() {
    int x = 42;
    BoxPtr<int> bp(&x);
    std::cout << *(bp.get()) << std::endl;

    DoubleBox<int> db(Box<int>(99));
    std::cout << db.get().get() << std::endl;
    return 0;
}
