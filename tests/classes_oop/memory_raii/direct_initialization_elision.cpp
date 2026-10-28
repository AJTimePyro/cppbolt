#include <cstdio>

// Every form of initialization below must run the constructor that g++ runs.
// C++17 guarantees that a prvalue of the class initializes the object
// directly, so no copy constructor runs for a prvalue.

class Value {
public:
    int number;
    Value(int v) : number(v) {
        std::printf("Value ctor %d\n", v);
    }
    Value(const Value& other) : number(other.number) {
        std::printf("Value copy ctor %d\n", other.number);
    }
};

class Pair {
public:
    int left;
    int right;
    Pair(int l, int r) : left(l), right(r) {
        std::printf("Pair ctor %d %d\n", l, r);
    }
};

Value make_value() {
    return Value(7); // A prvalue return never runs the copy constructor.
}

int main() {
    std::printf("-- direct initialization from a prvalue --\n");
    Value a(Value(1));
    std::printf("a.number = %d\n", a.number);

    std::printf("-- copy initialization from a prvalue --\n");
    Value b = Value(2);
    std::printf("b.number = %d\n", b.number);

    std::printf("-- direct initialization from an lvalue --\n");
    Value c(3);
    Value d(c);
    std::printf("d.number = %d\n", d.number);

    std::printf("-- direct initialization from a call --\n");
    Value e(make_value());
    std::printf("e.number = %d\n", e.number);

    std::printf("-- copy initialization from a call --\n");
    Value f = make_value();
    std::printf("f.number = %d\n", f.number);

    std::printf("-- direct initialization with two arguments --\n");
    Pair g(8, 9);
    std::printf("g.left = %d, g.right = %d\n", g.left, g.right);

    std::printf("-- copy initialization with two arguments --\n");
    Pair h = Pair(10, 11);
    std::printf("h.left = %d, h.right = %d\n", h.left, h.right);

    return 0;
}
