#include <stdio.h>

struct Target {
    int value;
    int getValue() const {
        return value;
    }
    void setValue(int v) {
        value = v;
    }
};

struct Proxy2 {
    Target* ptr;
    Proxy2(Target* p) : ptr(p) {}
    Target* operator->() {
        return ptr;
    }
};

struct Proxy1 {
    Proxy2 p2;
    Proxy1(Target* p) : p2(p) {}
    Proxy2& operator->() {
        return p2;
    }
};

struct SmartPtr {
    Proxy1 p1;
    SmartPtr(Target* p) : p1(p) {}
    Proxy1& operator->() {
        return p1;
    }
};

struct ValProxy1 {
    Proxy2 p2;
    ValProxy1(Target* p) : p2(p) {}
    Proxy2 operator->() {
        return p2;
    }
};

int main() {
    Target target;
    target.value = 100;

    // Direct pointer (1-hop proxy)
    Proxy2 p2(&target);
    printf("Proxy2 field: %d\n", p2->value);
    printf("Proxy2 method: %d\n", p2->getValue());

    // 2-hop proxy reference
    Proxy1 p1(&target);
    printf("Proxy1 field: %d\n", p1->value);
    printf("Proxy1 method: %d\n", p1->getValue());

    // 3-hop proxy reference
    SmartPtr sp(&target);
    printf("SmartPtr field: %d\n", sp->value);
    printf("SmartPtr method: %d\n", sp->getValue());

    // Mutation through 3-hop proxy
    sp->value = 250;
    printf("Mutated field: %d\n", sp->value);
    sp->setValue(350);
    printf("Mutated via method: %d\n", sp->getValue());

    // Value-returning proxy
    ValProxy1 vp(&target);
    printf("ValProxy1 field: %d\n", vp->value);
    printf("ValProxy1 method: %d\n", vp->getValue());

    return 0;
}
