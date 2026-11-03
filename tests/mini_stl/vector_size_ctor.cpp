#include <iostream>
#include <vector>

int main() {
    std::vector<int> v(3, 4);
    std::cout << "size: " << v.size() << std::endl;
    for (size_t i = 0; i < v.size(); ++i) {
        std::cout << v[i] << " ";
    }
    std::cout << std::endl;

    std::vector<int> v0(0, 42);
    std::cout << "v0 size: " << v0.size() << std::endl;

    return 0;
}
