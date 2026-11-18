#include <cstdio>

struct M {
    ~M() { std::printf("~M "); }
};

struct D {
    M m;
    ~D() { std::printf("~D "); return; }
};

int main() {
    {
        D d;
    }
    std::printf("\n");
    return 0;
}
