#include <iostream>
#include <utility>
#include <vector>

template <typename T>
using IntPair = std::pair<T, int>;

template <typename T>
using Vec = std::vector<T>;

int main() {
    IntPair<const char*> p("item", 10);
    Vec<int> v;
    v.push_back(p.second);
    std::cout << p.first << ": " << v[0] << std::endl;
    return 0;
}
