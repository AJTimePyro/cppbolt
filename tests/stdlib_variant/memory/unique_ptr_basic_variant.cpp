#include <cstdio>
#include <memory>
#include <utility>

struct Data {
    int x;
    int y;
    Data(int a, int b) : x(a), y(b) { printf("Data ctor: %d, %d\n", x, y); }
    ~Data() { printf("Data dtor: %d, %d\n", x, y); }
};

int main() {
    std::unique_ptr<Data> u1 = std::make_unique<Data>(10, 20);
    printf("u1: %d, %d\n", u1->x, u1->y);

    std::unique_ptr<Data> u2;
    u2 = std::move(u1);
    printf("u1 empty: %d, u2 has data: %d\n", !u1, bool(u2));
    printf("u2: %d, %d\n", u2->x, u2->y);

    Data* raw = u2.release();
    printf("after release: u2 has data: %d\n", bool(u2));
    printf("raw: %d, %d\n", raw->x, raw->y);
    delete raw;

    return 0;
}
