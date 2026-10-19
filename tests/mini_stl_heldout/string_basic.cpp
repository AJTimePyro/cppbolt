#include <iostream>
#include <string>

int main() {
    std::string a = "Web";
    std::string b = "Assembly";
    std::string c = a + "-" + b;
    std::cout << c << std::endl;
    std::cout << "len: " << c.length() << std::endl;
    std::cout << "alias: " << c.c_str() << std::endl;
    std::cout << "neq: " << (a == b) << std::endl;
    return 0;
}
