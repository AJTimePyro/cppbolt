#include <cstdio>

class Level0 {
public:
    virtual int query() { return 11; }
    virtual ~Level0() {}
};

class Level1 : public Level0 {
public:
    virtual int query() { return 22; }
};

class Level2 : public Level1 {
public:
    virtual int query() { return 33; }
};

void run_query(Level0& target) {
    std::printf("query result = %d\n", target.query());
}

void run_all(Level0& first, Level1& second, Level2& third) {
    run_query(first);
    run_query(second);
    run_query(third);
}

int main() {
    Level0 l0;
    Level1 l1;
    Level2 l2;

    run_all(l0, l1, l2);
    return 0;
}
