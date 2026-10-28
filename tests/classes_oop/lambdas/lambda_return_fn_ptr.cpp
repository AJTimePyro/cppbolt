#include <iostream>

typedef int (*BinaryOp)(int, int);

BinaryOp get_adder() {
    return [](int a, int b) { return a + b; };
}

int main() {
    BinaryOp op = get_adder();
    std::cout << op(20, 22) << std::endl;
    return 0;
}
