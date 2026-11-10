#include <cstdio>

struct Resource {
    int tag;
    Resource(int t = 0) : tag(t) {}
    Resource(const Resource& o) : tag(o.tag) { printf("res copy\n"); }
    Resource(Resource&& o) : tag(o.tag) { o.tag = -1; printf("res move\n"); }
    Resource& operator=(const Resource& o) {
        tag = o.tag;
        printf("res copy assign\n");
        return *this;
    }
    Resource& operator=(Resource&& o) {
        tag = o.tag;
        o.tag = -1;
        printf("res move assign\n");
        return *this;
    }
};

struct PairOfRes {
    Resource r1;
    Resource r2;
};

int main() {
    PairOfRes p1{Resource{11}, Resource{22}};
    printf("-- move pair --\n");
    PairOfRes p2 = static_cast<PairOfRes&&>(p1);
    printf("p2: %d %d, p1: %d %d\n", p2.r1.tag, p2.r2.tag, p1.r1.tag, p1.r2.tag);

    PairOfRes p3{Resource{99}, Resource{99}};
    printf("-- move assign pair --\n");
    p3 = static_cast<PairOfRes&&>(p2);
    printf("p3: %d %d, p2: %d %d\n", p3.r1.tag, p3.r2.tag, p2.r1.tag, p2.r2.tag);
    return 0;
}
