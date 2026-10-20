#include <iostream>
#include <string>

struct MessageWrapper {
    std::string payload;
    int tag;
    MessageWrapper(const std::string& p, int t) : payload(p), tag(t) {}
    MessageWrapper(const MessageWrapper& o) : payload(o.payload), tag(o.tag * 10) {}
    MessageWrapper& operator=(const MessageWrapper& o) {
        if (this != &o) {
            payload = o.payload;
            tag = o.tag * 20;
        }
        return *this;
    }
    ~MessageWrapper() {}
};

std::string transform_by_value(std::string str) {
    str.append("_processed");
    return str;
}

int main() {
    std::string text;
    std::cout << "init empty: " << text.empty() << " len: " << text.size() << std::endl;
    text.push_back('O');
    text.push_back('K');
    text.append("?");
    std::cout << "text: " << text << " len: " << text.size() << " empty: " << text.empty() << std::endl;

    std::cout << "index 0: " << text[0] << std::endl;
    std::string part = text.substr(0, 2);
    std::cout << "part: " << part << std::endl;
    std::cout << "find K: " << text.find("K") << " find missing: " << (text.find("404") == std::string::npos) << std::endl;

    std::string x = "cherry";
    std::string y = "date";
    std::string x2 = "cherry";
    std::cout << "cmp eq: " << (x == x2) << " cmp ne: " << (x != y) << " cmp lt: " << (x < y) << " cmp val: " << x.compare(y) << std::endl;

    text = text;
    std::cout << "self: " << text << std::endl;
    text += text;
    std::cout << "doubled: " << text << std::endl;
    std::string clone = text;
    clone[1] = 'Z';
    std::cout << "base: " << text << " clone: " << clone << std::endl;

    std::string out = transform_by_value(x);
    std::cout << "by_val_out: " << out << " base_x: " << x << std::endl;

    MessageWrapper m1("initial", 5);
    MessageWrapper m2 = m1;
    MessageWrapper m3("other", 7);
    m3 = m1;
    std::cout << "m1: " << m1.payload << " " << m1.tag << std::endl;
    std::cout << "m2: " << m2.payload << " " << m2.tag << std::endl;
    std::cout << "m3: " << m3.payload << " " << m3.tag << std::endl;

    std::string w1;
    std::string w2;
    std::string entry;
    std::cin >> w1 >> w2;
    std::cout << "tokens: <" << w1 << "> <" << w2 << ">" << std::endl;
    std::getline(std::cin, entry);
    std::getline(std::cin, entry);
    std::cout << "entry: <" << entry << ">" << std::endl;

    std::string large;
    for (int k = 0; k < 1000000; k++) {
        large.push_back((char)('z' - (k % 26)));
    }
    std::cout << "large: " << large.size() << " " << large[0] << " " << large[999999] << std::endl;

    text.clear();
    std::cout << "post clear: empty=" << text.empty() << " len=" << text.size() << std::endl;

    return 0;
}
