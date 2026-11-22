#include <cstdio>

struct Animal {
    int id;
    virtual int speak() {
        return id + 1;
    }
};

struct Dog : Animal {
    virtual int speak() override {
        return id + 100;
    }
};

struct Cat : Animal {
    virtual int speak() override {
        return id + 200;
    }
};

int main() {
    Dog dog;
    dog.id = 5;
    Cat cat;
    cat.id = 8;

    int (Animal::*pm_speak)() = &Animal::speak;

    Animal* p1 = &dog;
    Animal* p2 = &cat;

    std::printf("dog: %d\n", (p1->*pm_speak)());
    std::printf("cat: %d\n", (p2->*pm_speak)());
    std::printf("dog dot: %d\n", (dog.*pm_speak)());

    return 0;
}
