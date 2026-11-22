#include <iostream>
#include <map>
#include <string>

int main() {
    std::map<std::string, int> m;
    m["apple"] = 5;
    m["banana"] = 8;
    m["cherry"] = 12;

    std::cout << "apple: " << m["apple"] << std::endl;
    std::cout << "banana: " << m["banana"] << std::endl;
    std::cout << "cherry: " << m["cherry"] << std::endl;
    std::cout << "size: " << m.size() << std::endl;

    std::cout << "map traversal:" << std::endl;
    for (std::map<std::string, int>::iterator it = m.begin(); it != m.end(); ++it) {
        std::cout << "  " << it->first << ": " << it->second << std::endl;
    }
    return 0;
}
