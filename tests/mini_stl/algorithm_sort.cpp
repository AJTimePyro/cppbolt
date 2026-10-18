#include <iostream>
#include <algorithm>
#include <vector>

int main() {
    std::vector<int> v;
    v.push_back(50);
    v.push_back(10);
    v.push_back(40);
    v.push_back(20);
    v.push_back(30);

    std::sort(v.begin(), v.end());
    std::cout << "sorted:";
    for (size_t i = 0; i < v.size(); ++i) {
        std::cout << " " << v[i];
    }
    std::cout << std::endl;
    return 0;
}
