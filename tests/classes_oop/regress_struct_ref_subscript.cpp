#include <cstdio>

struct Container {
    int data[4];
    Container() {
        data[0] = 10;
        data[1] = 20;
        data[2] = 30;
        data[3] = 40;
    }
    int& operator[](int idx) {
        return data[idx];
    }
};

int get_element(Container& c, int idx) {
    return c[idx];
}

int main() {
    Container c;
    Container& ref = c;
    std::printf("ref[1]: %d\n", ref[1]);
    ref[2] = 999;
    std::printf("ref[2] updated: %d\n", ref[2]);
    std::printf("via helper[2]: %d\n", get_element(c, 2));
    return 0;
}
