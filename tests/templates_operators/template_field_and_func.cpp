#include <cstdio>

template <typename T>
struct Pair {
    T first;
    T second;
};

struct Container {
    Pair<double> p;
    int tag;
};

int consume(Pair<int> arg) {
    return arg.first + arg.second;
}

int main() {
    Pair<int> a;
    a.first = 10;
    a.second = 20;
    printf("%d\n", consume(a));

    Container c;
    c.p.first = 1.5;
    c.p.second = 2.5;
    c.tag = 99;
    printf("%.1f %.1f %d (size=%d)\n", c.p.first, c.p.second, c.tag, (int)sizeof(Container));
    return 0;
}
