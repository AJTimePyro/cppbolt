#include <cstdio>

class Member {
    int id;
public:
    Member(int i) : id(i) {
        std::printf("Member ctor %d\n", id);
    }
    ~Member() {
        std::printf("Member dtor %d\n", id);
    }
};

class Owner {
    Member m1;
    Member m2;
    Member m3;
public:
    Owner(int a, int b, int c) : m1(a), m2(b), m3(c) {
        std::printf("Owner ctor body\n");
    }
    ~Owner() {
        std::printf("Owner dtor body\n");
    }
};

int main() {
    std::printf("--- begin ---\n");
    {
        Owner o(1, 2, 3);
    }
    std::printf("--- end ---\n");
    return 0;
}
