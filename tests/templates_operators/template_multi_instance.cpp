#include <cstdio>

template <typename T>
class Box {
    T value;
public:
    Box(T val) : value(val) {}
    T get() const { return value; }
};

int main() {
    Box<int> bi(100);
    Box<double> bd(3.14);
    Box<char> bc('Z');
    std::printf("int: %d\n", bi.get());
    std::printf("double: %.2f\n", bd.get());
    std::printf("char: %c\n", bc.get());
    return 0;
}
