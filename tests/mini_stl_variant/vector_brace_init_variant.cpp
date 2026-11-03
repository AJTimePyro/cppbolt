#include <iostream>
#include <string>
#include <vector>

int main() {
    std::vector<std::string> words = {"alpha", "beta", "gamma"};
    std::cout << "words: " << words.size() << std::endl;
    for (size_t i = 0; i < words.size(); ++i) {
        std::cout << words[i] << " ";
    }
    std::cout << std::endl;

    std::vector<int> values{100, 200, 300};
    std::vector<int> copied({1, 2, 3, 4});
    std::cout << "values: " << values.size() << " " << copied.size() << std::endl;
    return 0;
}
