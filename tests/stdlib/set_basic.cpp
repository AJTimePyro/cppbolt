#include <iostream>
#include <set>

int main() {
    std::set<int> s;
    s.insert(30);
    s.insert(10);
    s.insert(20);
    s.insert(10); // duplicate

    std::cout << "size: " << s.size() << std::endl;
    std::cout << "count 10: " << s.count(10) << std::endl;
    std::cout << "count 40: " << s.count(40) << std::endl;

    std::cout << "set elements:";
    for (std::set<int>::iterator it = s.begin(); it != s.end(); ++it) {
        std::cout << " " << *it;
    }
    std::cout << std::endl;
    return 0;
}
