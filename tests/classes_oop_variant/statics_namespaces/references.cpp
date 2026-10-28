#include <cstdio>

void increment_pair(int& a, int& b) {
    a += 10;
    b += 20;
}

int current_setting = 42;
int& get_setting_ref() {
    return current_setting;
}

int main() {
    int p = 5;
    int q = 8;
    increment_pair(p, q);
    std::printf("pair: p=%d, q=%d\n", p, q);

    get_setting_ref() = 1000;
    std::printf("setting: %d\n", current_setting);
    return 0;
}
