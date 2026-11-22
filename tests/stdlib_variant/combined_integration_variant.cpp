#include <iostream>
#include <vector>
#include <string>
#include <algorithm>

struct Book {
    std::string title;
    int year;
    Book(std::string t = "", int y = 0) : title(t), year(y) {}
    bool operator<(const Book& o) const {
        return year < o.year;
    }
};

struct ByYearDesc {
    bool operator()(const Book& a, const Book& b) const {
        return a.year > b.year;
    }
};

int main() {
    std::vector<Book> library;
    library.push_back(Book("The Odyssey", 800));
    library.push_back(Book("Dune", 1965));
    library.push_back(Book("Foundation", 1951));

    std::sort(library.begin(), library.end());
    std::cout << "By year asc:" << std::endl;
    for (size_t i = 0; i < library.size(); ++i) {
        std::cout << library[i].year << ": " << library[i].title << std::endl;
    }

    ByYearDesc desc;
    std::sort(library.begin(), library.end(), desc);
    std::cout << "By year desc:" << std::endl;
    for (size_t i = 0; i < library.size(); ++i) {
        std::cout << library[i].year << ": " << library[i].title << std::endl;
    }

    std::reverse(library.begin(), library.end());
    std::cout << "Reversed:" << std::endl;
    for (size_t i = 0; i < library.size(); ++i) {
        std::cout << library[i].year << ": " << library[i].title << std::endl;
    }

    return 0;
}
