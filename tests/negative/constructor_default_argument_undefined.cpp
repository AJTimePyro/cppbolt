// The default argument of the constructor is analyzed when the constructor
// is selected, so a default argument that names an unknown variable is a
// diagnostic.
struct T {
    int value;
    T(int first, int second = missing_name) : value(first + second) {}
};

int main() {
    T t(1);
    return t.value;
}
