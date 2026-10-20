#include <cstdio>
#include <new>

struct Vec2 {
    int u;
    int v;
    Vec2(int u, int v) : u(u), v(v) {
        printf("Vec2 init: %d, %d\n", u, v);
    }
    ~Vec2() {
        printf("Vec2 destroy: %d, %d\n", u, v);
    }
};

char memory_pool[256];

int main() {
    Vec2* item = new (memory_pool) Vec2(100, 250);
    printf("item->u = %d, item->v = %d\n", item->u, item->v);
    return 0;
}
