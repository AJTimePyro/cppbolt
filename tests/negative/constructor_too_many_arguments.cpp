struct T {
    int value;
    T(int a, int b) : value(a + b) {}
};

int main() {
    T t(1, 2, 3);
    return t.value;
}
