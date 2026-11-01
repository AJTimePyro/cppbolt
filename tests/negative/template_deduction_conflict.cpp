template <typename T>
T my_min(T a, T b) {
    return (a < b) ? a : b;
}

int main() {
    my_min(10, 20.5);
    return 0;
}
