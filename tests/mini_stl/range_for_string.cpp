#include <iostream>
#include <string>

int main() {
    std::string s = "hello";
    for (char c : s) {
        std::cout << c << " ";
    }
    std::cout << std::endl;

    for (char& c : s) {
        if (c >= 'a' && c <= 'z') {
            c = c - 'a' + 'A';
        }
    }
    std::cout << s << std::endl;

    return 0;
}
