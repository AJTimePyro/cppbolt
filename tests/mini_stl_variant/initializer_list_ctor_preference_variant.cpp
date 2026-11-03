#include <iostream>
#include <vector>
#include <string>

int main() {
    std::vector<std::string> v1{"alpha", "beta"};
    std::cout << "v1 size: " << v1.size() << std::endl;
    for (size_t i = 0; i < v1.size(); ++i) {
        std::cout << v1[i] << " ";
    }
    std::cout << std::endl;

    std::vector<std::string> v2(2, "gamma");
    std::cout << "v2 size: " << v2.size() << std::endl;
    for (size_t i = 0; i < v2.size(); ++i) {
        std::cout << v2[i] << " ";
    }
    std::cout << std::endl;

    std::vector<std::string> v_empty{};
    std::cout << "v_empty size: " << v_empty.size() << std::endl;

    return 0;
}
