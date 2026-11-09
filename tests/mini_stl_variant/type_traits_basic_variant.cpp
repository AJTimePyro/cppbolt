#include <iostream>
#include <type_traits>

template <typename T, typename U>
void print_comparison() {
    if (std::is_same<T, U>::value) {
        std::cout << "same" << std::endl;
    } else {
        std::cout << "different" << std::endl;
    }
}

int main() {
    print_comparison<float, float>();
    print_comparison<int, const int>();
    std::remove_reference<const char*&>::type msg = "hello traits";
    std::cout << msg << std::endl;
    return 0;
}
