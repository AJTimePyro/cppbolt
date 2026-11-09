#include <cstdio>

struct MixedInit {
    int p = 5;
    int q{15};
};

struct Person {
    int age = 18;
    int score = 100;
    Person() : age(21) {}
    Person(int a, int s) : age(a), score(s) {}
};

int main() {
    MixedInit m;
    std::printf("m: %d %d\n", m.p, m.q);

    Person p1;
    Person p2(30, 95);
    std::printf("p1: %d %d\n", p1.age, p1.score);
    std::printf("p2: %d %d\n", p2.age, p2.score);

    return 0;
}
