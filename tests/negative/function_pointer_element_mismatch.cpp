void store(int* target) {
    *target = 1;
}

int main() {
    double value = 0.0;
    store(&value);
    return 0;
}
