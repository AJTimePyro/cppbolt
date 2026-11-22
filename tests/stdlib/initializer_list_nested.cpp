#include <initializer_list>
#include <string>
#include <vector>
#include <iostream>

int main() {
    std::vector<std::vector<int> > rows{{1, 2}, {3, 4}, {5}};
    std::cout << "rows: " << rows.size() << std::endl;
    for (size_t i = 0; i < rows.size(); ++i) {
        std::cout << "row " << i << ":";
        for (size_t j = 0; j < rows[i].size(); ++j) {
            std::cout << " " << rows[i][j];
        }
        std::cout << std::endl;
    }

    std::initializer_list<std::vector<int> > lists = {{7, 8}, {9}};
    std::cout << "lists: " << lists.size() << " " << (*lists.begin())[0] << std::endl;

    std::vector<std::vector<std::string> > words{
        {std::string("alpha"), std::string("beta")},
        {std::string("gamma")}};
    std::cout << "words: " << words.size() << " " << words[0][0] << " " << words[0][1]
              << " " << words[1][0] << std::endl;
    return 0;
}