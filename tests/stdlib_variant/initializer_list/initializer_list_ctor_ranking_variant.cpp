#include <initializer_list>
#include <iostream>

template <class T>
struct Ranked {
    int tag;
    Ranked(std::initializer_list<long> values) : tag(200) { (void)values; }
    Ranked(std::initializer_list<int> values) : tag(100) { (void)values; }
};

template <class T>
struct WithDefault {
    int tag;
    WithDefault() : tag(0) {}
    WithDefault(std::initializer_list<int> values) : tag(1 + (int)values.size()) { (void)values; }
};

int main() {
    Ranked<int> r{1};
    Ranked<char> c{2};
    WithDefault<int> d{};
    WithDefault<int> e{7, 8};
    std::cout << "ranked int: " << r.tag << std::endl;
    std::cout << "ranked char: " << c.tag << std::endl;
    std::cout << "default: " << d.tag << std::endl;
    std::cout << "list: " << e.tag << std::endl;
    return 0;
}