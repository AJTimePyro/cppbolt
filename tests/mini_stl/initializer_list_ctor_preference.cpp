#include <iostream>
#include <vector>

int main() {
    // v_brace{3, 4} must call vector(initializer_list<int>) -> 2 elements: 3, 4
    std::vector<int> v_brace{3, 4};
    std::cout << "v_brace size: " << v_brace.size() << std::endl;
    for (size_t i = 0; i < v_brace.size(); ++i) {
        std::cout << v_brace[i] << " ";
    }
    std::cout << std::endl;

    // v_paren(3, 4) must call vector(size_t, const int&) -> 3 elements: 4, 4, 4
    std::vector<int> v_paren(3, 4);
    std::cout << "v_paren size: " << v_paren.size() << std::endl;
    for (size_t i = 0; i < v_paren.size(); ++i) {
        std::cout << v_paren[i] << " ";
    }
    std::cout << std::endl;

    // v_empty{} must call default constructor -> 0 elements
    std::vector<int> v_empty{};
    std::cout << "v_empty size: " << v_empty.size() << std::endl;

    return 0;
}
