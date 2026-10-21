struct Other {
    int value;
};

class Counter {
public:
    int value;
    Counter() : value(0) {}
    void set(int v) { value = v; }
};

int main() {
    Counter counter;
    Other other;
    counter.set(other);
    return counter.value;
}
