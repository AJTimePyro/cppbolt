#include <iostream>
#include <vector>

int main() {
    std::vector<int> v;
    v.push_back(10);
    v.push_back(20);
    v.push_back(30);

    std::cout << "range-for:";
    for (int x : v) {
        std::cout << " " << x;
    }
    std::cout << std::endl;

    std::cout << "iterator:";
    for (std::vector<int>::iterator it = v.begin(); it != v.end(); ++it) {
        std::cout << " " << *it;
    }
    std::cout << std::endl;
    return 0;
}
