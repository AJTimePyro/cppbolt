#include <iostream>
#include <vector>
#include <algorithm>

bool descending(int a, int b) {
    return a > b;
}

int main() {
    std::vector<int> v;
    v.push_back(3);
    v.push_back(1);
    v.push_back(4);
    v.push_back(2);

    std::sort(v.begin(), v.end(), descending);

    std::cout << "sorted descending:";
    for (size_t i = 0; i < v.size(); ++i) {
        std::cout << " " << v[i];
    }
    std::cout << std::endl;
    return 0;
}
