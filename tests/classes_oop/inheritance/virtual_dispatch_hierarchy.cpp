#include <cstdio>

class BaseA {
public:
    virtual int get_code() { return 100; }
    virtual ~BaseA() {}
};

class MidB : public BaseA {
public:
    virtual int get_code() { return 200; }
};

class LeafC : public MidB {
public:
    virtual int get_code() { return 300; }
};

void test_call(BaseA& ref) {
    std::printf("ref get_code: %d\n", ref.get_code());
}

int main() {
    BaseA a;
    MidB b;
    LeafC c;

    test_call(a);
    test_call(b);
    test_call(c);
    return 0;
}
