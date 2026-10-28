#include <cstdio>

class Widget {
public:
    int val;
    Widget(int v) : val(v) {
        std::printf("Widget ctor %d\n", val);
    }
    ~Widget() {
        std::printf("Widget dtor %d\n", val);
    }
};

int main() {
    int* p = new int(42);
    std::printf("*p = %d\n", *p);
    delete p;

    Widget* w = new Widget(100);
    std::printf("w->val = %d\n", w->val);
    delete w;

    int* arr = new int[3];
    arr[0] = 7;
    arr[1] = 8;
    arr[2] = 9;
    std::printf("arr sum: %d\n", arr[0] + arr[1] + arr[2]);
    delete[] arr;
    return 0;
}
