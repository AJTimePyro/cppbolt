#include <iostream>

template <typename T>
struct Formatter {
    void print(T val) { std::cout << "generic: " << val << std::endl; }
};

template <>
struct Formatter<bool> {
    void print(bool val) { std::cout << "bool: " << (val ? "true" : "false") << std::endl; }
};

int main() {
    Formatter<int> fi;
    fi.print(42);
    Formatter<bool> fb;
    fb.print(true);
    return 0;
}
