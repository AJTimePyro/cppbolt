#include <cstdio>

struct Vector3D {
    int x;
    int y;
    int z;
};

void show_vector(Vector3D v) {
    std::printf("Vector3D(%d, %d, %d)\n", v.x, v.y, v.z);
}

int main() {
    Vector3D vec = {10, -20, 30};
    show_vector(vec);
    vec.x += 5;
    vec.y += 10;
    vec.z -= 15;
    show_vector(vec);
    return 0;
}
