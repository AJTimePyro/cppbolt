#include <cstdio>

class BaseClean {
public:
    BaseClean() { std::printf("BaseClean ctor\n"); }
    virtual ~BaseClean() { std::printf("BaseClean dtor\n"); }
};

class DerivedClean : public BaseClean {
    int* data;
public:
    DerivedClean() : data(new int(123)) {
        std::printf("DerivedClean ctor\n");
    }
    virtual ~DerivedClean() {
        std::printf("DerivedClean dtor\n");
        delete data;
    }
};

int main() {
    BaseClean* ptr = new DerivedClean();
    delete ptr;
    return 0;
}
