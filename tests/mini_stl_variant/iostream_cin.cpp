#include <iostream>
#include <string>

int main() {
    int num;
    std::string word;
    if (std::cin >> num >> word) {
        std::cout << "value: " << num * 3;
        if (num > 10) {
            std::cout << ", token: " << word;
        }
        std::cout << std::endl;
    }
    return 0;
}
