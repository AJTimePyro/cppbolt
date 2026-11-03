#include <initializer_list>
#include <iostream>
#include <string>
#include <vector>

std::vector<std::vector<int> > grid = {{1, 2}, {3}};
std::initializer_list<std::string> names = {std::string("alpha"),
                                            std::string("beta")};
std::vector<long> longs = {7L, 8L, 9L};

int main() {
    std::cout << "grid: " << grid.size() << " " << grid[0].size() << " "
              << grid[1].size() << std::endl;
    std::cout << "names: " << names.size() << " " << names.begin()->c_str()
              << " " << (names.begin() + 1)->c_str() << std::endl;
    std::cout << "longs: " << longs.size() << " " << longs[2] << std::endl;
    return 0;
}
