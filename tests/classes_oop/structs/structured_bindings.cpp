#include <cstdio>

struct Point {
    int x;
    int y;
};

struct Person {
    int id;
    int age;
    int score;
};

int main() {
    Point p;
    p.x = 10;
    p.y = 20;

    // By value: copies p
    auto [vx, vy] = p;
    vx = 99;
    std::printf("by_val: vx=%d vy=%d orig_x=%d\n", vx, vy, p.x);

    // By reference: aliases p
    auto& [rx, ry] = p;
    rx = 42;
    ry = 84;
    std::printf("by_ref: rx=%d ry=%d orig_x=%d orig_y=%d\n", rx, ry, p.x, p.y);

    // By const reference
    const auto& [cx, cy] = p;
    std::printf("by_const_ref: cx=%d cy=%d\n", cx, cy);

    // Multiple fields of aggregate
    Person person;
    person.id = 1;
    person.age = 30;
    person.score = 100;
    auto [pid, page, pscore] = person;
    std::printf("person: id=%d age=%d score=%d\n", pid, page, pscore);

    return 0;
}
