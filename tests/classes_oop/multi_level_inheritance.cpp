#include <cstdio>

class Animal {
public:
    virtual void speak() { std::printf("Animal sound\n"); }
};

class Mammal : public Animal {
public:
    virtual void speak() { std::printf("Mammal sound\n"); }
};

class Dog : public Mammal {
public:
    virtual void speak() { std::printf("Woof!\n"); }
};

int main() {
    Animal a;
    Mammal m;
    Dog d;

    Animal* p1 = &a;
    Animal* p2 = &m;
    Animal* p3 = &d;

    p1->speak();
    p2->speak();
    p3->speak();
    return 0;
}
