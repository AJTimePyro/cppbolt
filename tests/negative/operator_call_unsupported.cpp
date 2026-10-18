struct F {
    int operator()(int x) { return x + 1; }
};

int main() {
    F f;
    return f(1);
}
