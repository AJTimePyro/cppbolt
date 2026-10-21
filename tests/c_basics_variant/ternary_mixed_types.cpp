#include <cstdio>

int main() {
    bool take_int = false;
    int whole = 3;
    double fraction = 4.75;
    double result = take_int ? whole : fraction;
    std::printf("%.2f\n", result);
    return 0;
}
