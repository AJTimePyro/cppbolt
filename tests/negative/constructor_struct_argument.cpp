struct Other {
    int value;
};

struct T {
    int value;
    T(int v) : value(v) {}
};

int main() {
    Other other;
    T t(other);
    return t.value;
}
