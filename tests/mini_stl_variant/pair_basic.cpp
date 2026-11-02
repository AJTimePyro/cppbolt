#include <iostream>
#include <utility>

int main() {
    std::pair<int, int> p1 = std::make_pair(42, 100);
    std::pair<double, double> p2(p1);
    if (p2.first == 42.0 && p2.second == 100.0) {
        std::cout << "converted pair: " << p2.first << ", " << p2.second << std::endl;
    }
    return 0;
}
