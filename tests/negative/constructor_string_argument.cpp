struct T {
    int value;
    T(int v) : value(v) {}
};

int main() {
    T t("hello");
    return t.value;
}
