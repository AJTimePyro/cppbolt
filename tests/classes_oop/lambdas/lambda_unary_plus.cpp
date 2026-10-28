#include <iostream>

int main() {
    auto fp = +[](int x) { return x * 3; };
    std::cout << sizeof(fp) << " " << fp(14) << std::endl;
    return 0;
}
