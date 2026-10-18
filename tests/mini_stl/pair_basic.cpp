#include <iostream>
#include <utility>
#include <string>

int main() {
    std::pair<int, std::string> p = std::make_pair(1, std::string("one"));
    std::cout << "first: " << p.first << ", second: " << p.second << std::endl;
    return 0;
}
