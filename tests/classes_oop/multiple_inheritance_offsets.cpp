#include <cstdio>

struct Base1 {
    int b1_a;
    int b1_b;
    int get_b1() { return b1_a + b1_b; }
};

struct Base2 {
    int b2_a;
    int b2_b;
    int get_b2() { return b2_a * 10 + b2_b; }
};

struct Base3 {
    int b3_a;
    int b3_b;
    int get_b3() { return b3_a * 100 + b3_b; }
};

struct Derived : Base1, Base2, Base3 {
    int d_a;
    int d_b;
    int get_all() {
        return b1_a + b2_a + b3_a + d_a;
    }
};

void check_b2(Base2* p, Base2& r) {
    std::printf("check_b2: ptr=%d ref=%d\n", p->get_b2(), r.get_b2());
}

void check_b3(Base3* p, Base3& r) {
    std::printf("check_b3: ptr=%d ref=%d\n", p->get_b3(), r.get_b3());
}

int main() {
    Derived d;
    d.b1_a = 1; d.b1_b = 2;
    d.b2_a = 3; d.b2_b = 4;
    d.b3_a = 5; d.b3_b = 6;
    d.d_a = 7;  d.d_b = 8;

    std::printf("direct: b1=%d b2=%d b3=%d all=%d\n",
                d.get_b1(), d.get_b2(), d.get_b3(), d.get_all());

    Base1* p1 = &d;
    Base2* p2 = &d;
    Base3* p3 = &d;

    std::printf("ptrs: b1=%d b2=%d b3=%d\n",
                p1->get_b1(), p2->get_b2(), p3->get_b3());

    check_b2(&d, d);
    check_b3(&d, d);

    Derived* from_b2 = static_cast<Derived*>(p2);
    Derived* from_b3 = static_cast<Derived*>(p3);

    std::printf("downcast: from_b2=%d from_b3=%d\n",
                from_b2->get_all(), from_b3->get_all());

    Derived* null_d = nullptr;
    Base2* null_b2 = null_d;
    Base3* null_b3 = static_cast<Base3*>(null_d);
    Derived* back_null = static_cast<Derived*>(null_b2);
    std::printf("null: b2=%d b3=%d back=%d\n",
                null_b2 == nullptr ? 1 : 0,
                null_b3 == nullptr ? 1 : 0,
                back_null == nullptr ? 1 : 0);

    return 0;
}
