#include <cstdio>
#include <new>

struct Resource {
    int id;
    Resource(int id) : id(id) {
        printf("Resource ctor %d\n", id);
    }
    ~Resource() {
        printf("Resource dtor %d\n", id);
    }
};

char buf[64];

int main() {
    Resource* r = new (buf) Resource(42);
    printf("Created resource %d\n", r->id);
    r->~Resource();
    printf("Explicit dtor called\n");
    return 0;
}
