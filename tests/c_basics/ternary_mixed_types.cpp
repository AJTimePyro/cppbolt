#include <cstdio>

int main() {
    bool condition = true;
    double value = 2.5;
    double result = !condition ? 1 : value;
    std::printf("%.2f\n", result);
    return 0;
}
