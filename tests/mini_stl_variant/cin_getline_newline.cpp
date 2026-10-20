#include <iostream>
#include <string>
#include <iomanip>

int main() {
    std::ios_base::sync_with_stdio(false);
    std::cin.tie(nullptr);

    int first = 0;
    int second = 0;
    std::cin >> first >> second;
    std::cout << "Sum: " << first + second << std::endl;

    std::string empty_line;
    std::getline(std::cin, empty_line);
    std::cout << "After numbers: " << empty_line.length() << std::endl;

    std::string line;
    std::getline(std::cin, line);
    std::cout << "Line: [" << line << "]" << std::endl;

    int ch = std::cin.peek();
    std::cout << "Peek: " << (char)ch << std::endl;

    std::cin.ignore(2);
    std::string tail;
    std::getline(std::cin, tail);
    std::cout << "Tail: [" << tail << "]" << std::endl;

    std::cout << std::setfill('.') << std::setw(6) << 77 << std::endl;
    std::cout << "eof: " << (std::cin.eof() ? 1 : 0) << std::endl;
    return 0;
}
