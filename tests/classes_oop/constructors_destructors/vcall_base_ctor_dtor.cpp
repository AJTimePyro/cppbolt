#include <cstdio>

class Base {
public:
    virtual void action() {
        std::printf("Base action\n");
    }
    Base() {
        std::printf("Base ctor before action\n");
        action();
    }
    virtual ~Base() {
        std::printf("Base dtor before action\n");
        action();
    }
};

class Derived : public Base {
public:
    void action() override {
        std::printf("Derived action\n");
    }
    Derived() {
        std::printf("Derived ctor before action\n");
        action();
    }
    ~Derived() override {
        std::printf("Derived dtor before action\n");
        action();
    }
};

int main() {
    std::printf("--- Constructing Derived ---\n");
    Derived d;
    std::printf("--- Destructing Derived ---\n");
    return 0;
}
