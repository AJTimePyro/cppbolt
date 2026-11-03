#include <initializer_list>
#include <iostream>

struct Ranked {
    int tag;
    Ranked(std::initializer_list<long> values) : tag(200) { (void)values; }
    Ranked(std::initializer_list<int> values) : tag(100) { (void)values; }
};

struct Floating {
    int tag;
    Floating(std::initializer_list<int> values) : tag(1) { (void)values; }
    Floating(std::initializer_list<double> values) : tag(2) { (void)values; }
};

struct Empty {
    int count;
    Empty(std::initializer_list<int> values) : count((int)values.size()) {}
};

int main() {
    Ranked r{1};
    Floating f{1.5};
    Empty e{};
    std::cout << "ranked: " << r.tag << std::endl;
    std::cout << "floating: " << f.tag << std::endl;
    std::cout << "empty: " << e.count << std::endl;
    return 0;
}