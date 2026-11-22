#include <iostream>

void show(const char* label, double value) {
    std::cout << label << ": " << value << std::endl;
}

int main() {
    show("f", 1.5f);
    show("d", 0.125);
    show("quarter", 0.25);
    show("hundred", 42.0);
    show("tiny", 1e-3);
    show("neg", -0.007);
    show("big", 98765.4321);
    show("sci", 1e5);
    show("e", 2.718281828);
    show("near", 9.999);
    return 0;
}
