#include <iostream>
#include <string>

int main() {
    std::string s1 = "Hello";
    std::string s2 = "World";
    std::string s3 = s1 + ", " + s2 + "!";
    std::cout << s3 << std::endl;
    std::cout << "length: " << s3.length() << std::endl;
    std::cout << "c_str: " << s3.c_str() << std::endl;
    std::cout << "equal: " << (s1 == "Hello") << std::endl;
    return 0;
}
