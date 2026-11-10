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
    std::remove_reference<const char*&>::type msg1 = "hello traits";
    std::remove_reference_t<const char*&> msg2 = "alias traits";
    std::remove_reference_t<int&> val = 42;
    std::cout << msg1 << " " << msg2 << " " << val << std::endl;
    return 0;
}
