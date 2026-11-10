#include <cstdio>

struct Resource {
    int id;
};

void inspect(Resource&& r) {
    printf("rvalue resource: %d\n", r.id);
}

int main() {
    inspect(Resource{101});
    int&& r = 42 + 8;
    printf("rref int: %d\n", r);
    return 0;
}
