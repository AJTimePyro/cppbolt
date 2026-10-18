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

int main() {
    Level0 l0;
    Level1 l1;
    Level2 l2;

    run_query(l0);
    run_query(l1);
    run_query(l2);
    return 0;
}
