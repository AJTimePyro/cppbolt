struct T {
    int value;
    T(int a, int b) : value(a + b) {}
};

int main() {
    T t(1);
    return t.value;
}
