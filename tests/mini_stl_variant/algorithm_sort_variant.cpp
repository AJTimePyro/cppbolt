#include <iostream>
#include <algorithm>
#include <vector>

struct Descending {
    bool operator()(int a, int b) const {
        return a > b;
    }
};

int main() {
    std::vector<int> nums;
    nums.push_back(3);
    nums.push_back(1);
    nums.push_back(4);
    nums.push_back(1);
    nums.push_back(5);
    nums.push_back(9);

    Descending desc;
    std::sort(nums.begin(), nums.end(), desc);

    std::cout << "desc:";
    for (int i = 0; i < nums.size(); ++i) {
        std::cout << " " << nums[i];
    }
    std::cout << std::endl;

    std::sort(nums.begin(), nums.end());
    std::cout << "asc:";
    for (int i = 0; i < nums.size(); ++i) {
        std::cout << " " << nums[i];
    }
    std::cout << std::endl;

    std::reverse(nums.begin(), nums.end());
    std::cout << "reversed:";
    for (int i = 0; i < nums.size(); ++i) {
        std::cout << " " << nums[i];
    }
    std::cout << std::endl;

    std::cout << "min: " << std::min(10, 20) << " max: " << std::max(10, 20) << std::endl;

    return 0;
}
