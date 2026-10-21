#include <cstdio>

class Parent {
public:
    int amount;
    Parent() : amount(30) {}
};

class Child : public Parent {
public:
    int amount;
    Child() : amount(80) {}
};

int main() {
    Child child;
    Parent& parent = child;
    std::printf("%d %d\n", child.amount, parent.amount);
    child.amount = 90;
    std::printf("%d %d\n", child.amount, parent.amount);
    return 0;
}
