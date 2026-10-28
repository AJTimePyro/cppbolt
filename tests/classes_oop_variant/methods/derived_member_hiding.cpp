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

void set_amount(Child& child, int amount) { child.amount = amount; }

int main() {
    Child child;
    Parent& parent = child;
    std::printf("%d %d\n", child.amount, parent.amount);
    set_amount(child, 90);
    std::printf("%d %d\n", child.amount, parent.amount);
    return 0;
}
