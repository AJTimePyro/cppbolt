#include <initializer_list>
#include <vector>
#include <iostream>

int main() {
    std::vector<std::vector<std::vector<int> > > cube{{{1}, {2, 3}}, {{4, 5}}};
    std::cout << "cube: " << cube.size() << " " << cube[0].size() << " " << cube[1].size()
              << " " << cube[0][1][1] << " " << cube[1][0][1] << std::endl;

    std::initializer_list<std::initializer_list<int> > nested = {{1, 2}, {3}};
    int total = 0;
    for (std::initializer_list<int> row : nested) {
        for (int value : row) {
            total += value;
        }
    }
    std::cout << "nested: " << nested.size() << " " << total << std::endl;
    return 0;
}