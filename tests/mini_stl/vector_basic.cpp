#include <iostream>
#include <vector>

int main() {
    std::vector<int> v;
    std::cout << "empty: " << v.empty() << std::endl;
    v.push_back(10);
    v.push_back(20);
    v.push_back(30);

    std::cout << "size: " << v.size() << std::endl;
    std::cout << "v[0]: " << v[0] << ", v[2]: " << v[2] << std::endl;

    v.pop_back();
    std::cout << "after pop size: " << v.size() << std::endl;

    v.clear();
    std::cout << "after clear size: " << v.size() << std::endl;
    return 0;
}
