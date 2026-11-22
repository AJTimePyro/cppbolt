#include <iostream>
#include <vector>
#include <string>

int main() {
    std::vector<std::string> v(4, "item");
    std::cout << "size: " << v.size() << std::endl;
    for (size_t i = 0; i < v.size(); ++i) {
        std::cout << v[i] << " ";
    }
    std::cout << std::endl;

    std::vector<char> vc(5, 'z');
    std::cout << "vc size: " << vc.size() << std::endl;
    for (size_t i = 0; i < vc.size(); ++i) {
        std::cout << vc[i];
    }
    std::cout << std::endl;

    return 0;
}
