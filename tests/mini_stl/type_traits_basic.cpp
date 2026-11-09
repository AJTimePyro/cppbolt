#include <iostream>
#include <type_traits>

int main() {
    std::cout << std::is_same<int, int>::value << std::endl;
    std::cout << std::is_same<int, double>::value << std::endl;
    std::remove_reference<int&>::type x = 100;
    std::cout << x << std::endl;
    std::cout << std::true_type::value << " " << std::false_type::value << std::endl;
    return 0;
}
