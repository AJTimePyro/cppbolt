#include <cstdio>

class Vehicle {
protected:
    int wheels;
public:
    Vehicle(int w) : wheels(w) {
        std::printf("Vehicle ctor: %d wheels\n", wheels);
    }
    ~Vehicle() {
        std::printf("Vehicle dtor: %d wheels\n", wheels);
    }
    void show_wheels() {
        std::printf("wheels count: %d\n", wheels);
    }
};

class Truck : public Vehicle {
    int cargo_tons;
public:
    Truck(int w, int tons) : Vehicle(w), cargo_tons(tons) {
        std::printf("Truck ctor: %d tons\n", cargo_tons);
    }
    ~Truck() {
        std::printf("Truck dtor: %d tons\n", cargo_tons);
    }
    void show_specs() {
        show_wheels();
        std::printf("cargo: %d tons\n", cargo_tons);
    }
};

int main() {
    Truck t(6, 15);
    t.show_specs();
    return 0;
}
