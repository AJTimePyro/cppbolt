#include <iostream>

template <typename T>
struct TypeDesc {
    static const char* name() { return "unknown"; }
    static int size() { return sizeof(T); }
};

template <>
struct TypeDesc<int> {
    static const char* name() { return "int"; }
    static int size() { return 4; }
};

template <>
struct TypeDesc<double> {
    static const char* name() { return "double"; }
    static int size() { return 8; }
};

int main() {
    std::cout << TypeDesc<char>::name() << std::endl;
    std::cout << TypeDesc<int>::name() << ": " << TypeDesc<int>::size() << std::endl;
    std::cout << TypeDesc<double>::name() << ": " << TypeDesc<double>::size() << std::endl;
    return 0;
}
