#include <iostream>
#include <string>

struct SensorState {
    int code;
    SensorState(int c) : code(c) {}
    explicit operator bool() const {
        return code >= 100;
    }
};

int main() {
    // for (; cin >> val; ) with different accumulator
    int val;
    int product = 1;
    for (; std::cin >> val;) {
        if (val == -1) break;
        product = product * val;
    }
    std::cout << "product: " << product << std::endl;

    // (cin >> val) ? a : b
    int outcome = (std::cin >> val) ? (val + 50) : 0;
    std::cout << "ternary_outcome: " << outcome << std::endl;

    // if (!(cin >> val))
    if (!(std::cin >> val)) {
        std::cout << "extraction rejected" << std::endl;
        std::cin.clear();
    } else {
        std::cout << "extraction accepted: " << val << std::endl;
    }

    // while (getline(cin, text))
    std::string text;
    int records = 0;
    while (std::getline(std::cin, text)) {
        records++;
        std::cout << "record #" << records << ": " << text << std::endl;
    }
    std::cout << "records processed: " << records << std::endl;

    // custom struct contextual bool
    SensorState s_active(150);
    SensorState s_idle(50);
    if (s_active) {
        std::cout << "active sensor detected" << std::endl;
    }
    if (!s_idle) {
        std::cout << "idle sensor bypassed" << std::endl;
    }
    std::cout << "sensor cond: " << (s_idle ? 999 : 333) << std::endl;

    return 0;
}
