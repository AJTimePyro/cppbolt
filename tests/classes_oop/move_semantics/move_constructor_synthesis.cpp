#include <cstdio>

struct Simple {
    int x;
    int y;
};

struct Tracker {
    int id;
    Tracker(int i = 0) : id(i) {}
    Tracker(const Tracker& o) : id(o.id) { printf("copy ctor\n"); }
    Tracker(Tracker&& o) : id(o.id) { o.id = 0; printf("move ctor\n"); }
    Tracker& operator=(const Tracker& o) {
        id = o.id;
        printf("copy assign\n");
        return *this;
    }
    Tracker& operator=(Tracker&& o) {
        id = o.id;
        o.id = 0;
        printf("move assign\n");
        return *this;
    }
};

struct Composite {
    Tracker t;
};

int main() {
    Simple s1{1, 2};
    Simple s2 = static_cast<Simple&&>(s1);
    printf("s2: %d %d\n", s2.x, s2.y);

    Composite c1{Tracker{42}};
    printf("-- moving composite --\n");
    Composite c2 = static_cast<Composite&&>(c1);
    printf("c2 id: %d, c1 id: %d\n", c2.t.id, c1.t.id);

    Composite c3{Tracker{100}};
    printf("-- move assigning composite --\n");
    c3 = static_cast<Composite&&>(c2);
    printf("c3 id: %d, c2 id: %d\n", c3.t.id, c2.t.id);
    return 0;
}
