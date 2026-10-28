int main() {
    int x = 1;
    [x, =]() { return x; }();
    return 0;
}
