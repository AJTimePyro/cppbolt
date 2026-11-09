#include <iostream>

template <typename T>
struct Inspector {
    static void check() { std::cout << "value type" << std::endl; }
};

template <typename T>
struct Inspector<T*> {
    static void check() { std::cout << "pointer type" << std::endl; }
};

template <typename T>
struct Inspector<const T*> {
    static void check() { std::cout << "const pointer type" << std::endl; }
};

int main() {
    Inspector<int>::check();
    Inspector<int*>::check();
    Inspector<const int*>::check();
    return 0;
}
