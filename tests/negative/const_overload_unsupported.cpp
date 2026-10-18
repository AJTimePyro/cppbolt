struct S {
    int value() const & { return 1; }
};

int value() {
    return 1;
}

int main() {
    return value();
}
