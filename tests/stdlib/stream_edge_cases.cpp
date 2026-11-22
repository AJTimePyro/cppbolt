#include <iostream>
#include <string>

int main() {
    int num = 0;
    std::cin >> num;
    std::cout << "num: " << num << " eof: " << std::cin.eof() << " fail: " << std::cin.fail() << std::endl;

    int p = std::cin.peek();
    std::cout << "peek: " << (char)p << " eof: " << std::cin.eof() << " fail: " << std::cin.fail() << std::endl;

    char c = '?';
    std::cin.get(c);
    std::cout << "get: " << c << " gcount: " << std::cin.gcount() << " eof: " << std::cin.eof() << " fail: " << std::cin.fail() << std::endl;

    std::string s;
    std::getline(std::cin, s);
    std::cout << "getline1: [" << s << "] eof: " << std::cin.eof() << " fail: " << std::cin.fail() << std::endl;

    std::string s2;
    std::getline(std::cin, s2);
    std::cout << "getline2: [" << s2 << "] eof: " << std::cin.eof() << " fail: " << std::cin.fail() << std::endl;

    std::cin.clear();
    int ch = std::cin.get();
    std::cout << "after clear get: " << ch << " eof: " << std::cin.eof() << " fail: " << std::cin.fail() << std::endl;

    return 0;
}
