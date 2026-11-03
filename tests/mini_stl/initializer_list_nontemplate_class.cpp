#include <initializer_list>
#include <vector>
#include <iostream>

struct Sizes {
    int count;
    Sizes(std::initializer_list<int> values) : count((int)values.size()) {}
};

typedef std::initializer_list<int> IntList;

struct Count {
    int count;
    Count(IntList values) : count((int)values.size()) {}
};

struct Elements {
    int count;
    Elements(std::vector<int> values) : count((int)values.size()) {}
};

int main() {
    Sizes s{4, 5, 6};
    Count c{7, 8};
    std::vector<int> v;
    v.push_back(1);
    v.push_back(2);
    v.push_back(3);
    Elements e(v);
    std::cout << "sizes: " << s.count << std::endl;
    std::cout << "count: " << c.count << std::endl;
    std::cout << "elements: " << e.count << std::endl;
    return 0;
}