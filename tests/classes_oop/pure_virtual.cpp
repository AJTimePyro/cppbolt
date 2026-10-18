#include <cstdio>

class AbstractSpeaker {
public:
    virtual void speak() = 0;
    virtual ~AbstractSpeaker() {}
};

class Human : public AbstractSpeaker {
public:
    virtual void speak() {
        std::printf("Hello!\n");
    }
};

class Robot : public AbstractSpeaker {
public:
    virtual void speak() {
        std::printf("Beep boop!\n");
    }
};

void make_sound(AbstractSpeaker* s) {
    s->speak();
}

int main() {
    Human h;
    Robot r;
    make_sound(&h);
    make_sound(&r);
    return 0;
}
