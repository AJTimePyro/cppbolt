#include <iostream>
#include <string>

int main() {
    int num;
    std::string word;
    if (std::cin >> num >> word) {
        std::cout << "Read num: " << num * 2 << ", word: " << word << std::endl;
    }
    return 0;
}
