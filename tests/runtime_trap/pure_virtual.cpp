#include <cstdio>

struct Base {
    Base() {
        this->helper();
    }
    void helper() {
        this->action();
    }
    virtual void action() = 0;
};

struct Derived : Base {
    Derived() {}
    void action() {
        printf("Derived action\n");
    }
};

int main() {
    Derived d;
    return 0;
}
