#include <iostream>
#include <string>
#include <iomanip>

int main() {

    std::ios_base::sync_with_stdio(false);
    std::cin.tie(nullptr);

    int n = 0;
    std::cin >> n;
    std::cout << "Read number: " << n << std::endl;

    // Test 1: Without ignore(), getline reads the empty string caused by the newline trailing after `cin >> n`!
    std::string empty_line;
    std::getline(std::cin, empty_line);
    std::cout << "Direct getline length: " << empty_line.length() << std::endl;

    // Test 2: Now getline reads the first line!
    std::string first_line;
    std::getline(std::cin, first_line);
    std::cout << "First line: [" << first_line << "]" << std::endl;

    // Test 3: Test peek
    int ch = std::cin.peek();
    std::cout << "Peek next char: " << (char)ch << std::endl;

    // Test 4: Use ignore to skip that char, then getline reads the rest
    std::cin.ignore(1);
    std::string rest;
    std::getline(std::cin, rest);
    std::cout << "Rest after ignore: [" << rest << "]" << std::endl;

    // Test 5: Manipulators setw and setfill
    std::cout << std::setfill('*') << std::setw(8) << 123 << std::endl;

    // Test 6: Manipulator setprecision and fixed
    std::cout << std::fixed << std::setprecision(2) << 3.14159 << std::endl;

    // Test 7: eof and fail
    std::cout << "cin eof: " << (std::cin.eof() ? 1 : 0) << ", fail: " << (std::cin.fail() ? 1 : 0) << std::endl;

    return 0;
}
