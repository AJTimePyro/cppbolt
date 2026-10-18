struct Counter {
    int value;
    Counter& operator++() { ++value; return *this; }
    Counter operator++(int) { Counter old = *this; ++value; return old; }
};

int main() {
    Counter c{0};
    c++;
    return c.value;
}
