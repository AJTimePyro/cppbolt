#include <cstdio>

class Payload {
public:
    int tag;
    Payload(int t) : tag(t) {
        std::printf("Payload ctor %d\n", tag);
    }
    Payload(const Payload& other) : tag(other.tag + 50) {
        std::printf("Payload copy ctor %d\n", tag);
    }
    ~Payload() {
        std::printf("Payload dtor %d\n", tag);
    }
};

Payload make_payload(int t) {
    return Payload(t);
}

void process_payload(Payload p) {
    std::printf("process_payload tag: %d\n", p.tag);
}

Payload forward_payload(int t) {
    return make_payload(t);
}

int main() {
    std::printf("--- start prvalue elision ---\n");
    Payload p = forward_payload(500);
    std::printf("p.tag = %d\n", p.tag);

    std::printf("--- arg prvalue elision ---\n");
    process_payload(Payload(800));

    std::printf("--- done prvalue elision ---\n");
    return 0;
}
