#include <iostream>
#include <string>

struct Flag {
    int val;
    Flag(int v) : val(v) {}
    explicit operator bool() const {
        return val != 0;
    }
};

int main() {
    // 1. for (; cin >> x; )
    int x;
    int sum = 0;
    for (; std::cin >> x;) {
        sum += x;
        if (x == 999) break;
    }
    std::cout << "sum: " << sum << std::endl;

    // 2. (cin >> x) ? a : b
    int res = (std::cin >> x) ? (x * 10) : -1;
    std::cout << "ternary: " << res << std::endl;

    // 3. if (!(cin >> x))
    if (!(std::cin >> x)) {
        std::cout << "not failed as expected" << std::endl;
        std::cin.clear();
    } else {
        std::cout << "not succeeded: " << x << std::endl;
    }

    // 4. while (getline(cin, s))
    std::string s;
    int line_count = 0;
    while (std::getline(std::cin, s)) {
        line_count++;
        std::cout << "line " << line_count << ": " << s << std::endl;
    }
    std::cout << "total lines: " << line_count << std::endl;

    // 5. custom struct contextual bool
    Flag f1(1);
    Flag f0(0);
    if (f1) {
        std::cout << "f1 true" << std::endl;
    }
    if (!f0) {
        std::cout << "f0 false" << std::endl;
    }
    std::cout << "flag ternary: " << (f1 ? 100 : 200) << std::endl;

    return 0;
}
