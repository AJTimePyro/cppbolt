#include <initializer_list>

struct OnlyInt {
    OnlyInt(std::initializer_list<int> values) { (void)values; }
};

int main() {
    OnlyInt x{1.5};
    return 0;
}