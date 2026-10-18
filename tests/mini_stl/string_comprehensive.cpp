#include <iostream>
#include <string>

struct Container {
    std::string text;
    int id;
    Container(const std::string& s, int i) : text(s), id(i) {}
    Container(const Container& other) : text(other.text), id(other.id + 100) {}
    Container& operator=(const Container& other) {
        if (this != &other) {
            text = other.text;
            id = other.id + 200;
        }
        return *this;
    }
    ~Container() {}
};

std::string pass_and_return_by_val(std::string s) {
    s += "_suffix";
    return s;
}

int main() {
    std::string s;
    std::cout << "empty: " << s.empty() << " size: " << s.size() << std::endl;
    s.push_back('H');
    s.push_back('i');
    s.append("!");
    std::cout << "str: " << s << " size: " << s.size() << " empty: " << s.empty() << std::endl;

    std::cout << "elem 1: " << s[1] << std::endl;
    std::string sub = s.substr(1, 2);
    std::cout << "substr: " << sub << std::endl;
    std::cout << "find i: " << s.find("i") << " find z: " << (s.find("z") == std::string::npos) << std::endl;

    std::string a = "apple";
    std::string b = "banana";
    std::string a2 = "apple";
    std::cout << "eq: " << (a == a2) << " ne: " << (a != b) << " lt: " << (a < b) << " cmp: " << a.compare(b) << std::endl;

    s = s;
    std::cout << "self-assign: " << s << std::endl;
    s += s;
    std::cout << "s+=s: " << s << std::endl;
    std::string copy = s;
    copy[0] = 'X';
    std::cout << "orig: " << s << " copy: " << copy << std::endl;

    std::string res = pass_and_return_by_val(a);
    std::cout << "byval: " << res << " orig_a: " << a << std::endl;

    Container c1("hello", 1);
    Container c2 = c1;
    Container c3("world", 3);
    c3 = c1;
    std::cout << "c1: " << c1.text << " " << c1.id << std::endl;
    std::cout << "c2: " << c2.text << " " << c2.id << std::endl;
    std::cout << "c3: " << c3.text << " " << c3.id << std::endl;

    std::string word1;
    std::string word2;
    std::string line;
    std::cin >> word1 >> word2;
    std::cout << "words: [" << word1 << "] [" << word2 << "]" << std::endl;
    std::getline(std::cin, line);
    std::getline(std::cin, line);
    std::cout << "line: [" << line << "]" << std::endl;

    std::string big;
    for (int i = 0; i < 1000000; i++) {
        big.push_back((char)('a' + (i % 26)));
    }
    std::cout << "big: " << big.size() << " " << big[0] << " " << big[999999] << std::endl;

    s.clear();
    std::cout << "after clear: empty=" << s.empty() << " size=" << s.size() << std::endl;

    return 0;
}
