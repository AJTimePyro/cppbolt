#include <iostream>
#include <array>

int main() {
    std::array<int, 5> a{10, 20, 30, 40, 50};
    std::cout << "before: " << a[0] << " " << a[4] << std::endl;
    a.fill(9);
    std::cout << "after: " << a[0] << " " << a[2] << " empty: " << a.empty() << std::endl;
    return 0;
}
