#include <iostream>
#include <algorithm>
#include <vector>

int main() {
    int a = 15;
    int b = 42;
    std::cout << "min: " << std::min(a, b) << std::endl;
    std::cout << "max: " << std::max(a, b) << std::endl;

    std::swap(a, b);
    std::cout << "after swap: a=" << a << ", b=" << b << std::endl;

    std::vector<int> v;
    v.push_back(1);
    v.push_back(2);
    v.push_back(3);
    std::reverse(v.begin(), v.end());
    std::cout << "reversed: " << v[0] << " " << v[1] << " " << v[2] << std::endl;
    return 0;
}
