#include <stdio.h>

struct Base1 {
    int x;
    int get_x() { return x; }
};

struct Base2 {
    int y;
    int get_y() { return y; }
};

struct Base3 {
    int z;
    int get_z() { return z; }
};

template <typename... Bases>
struct Multi : Bases... {
    int total() {
        return this->x + this->y + this->z;
    }
};

int main() {
    Multi<Base1, Base2, Base3> m;
    m.x = 10;
    m.y = 20;
    m.z = 30;
    printf("Base methods: %d %d %d\n", m.get_x(), m.get_y(), m.get_z());
    printf("Total: %d\n", m.total());
    return 0;
}
