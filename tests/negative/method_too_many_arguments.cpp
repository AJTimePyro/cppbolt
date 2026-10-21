class Counter {
public:
    int value;
    Counter() : value(0) {}
    void set(int v) { value = v; }
};

int main() {
    Counter counter;
    counter.set(1, 2);
    return counter.value;
}
