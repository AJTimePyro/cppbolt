struct S {
    int x = later();
};

int later() {
    return 7;
}

int main() {
    S s;
    return s.x - 7;
}
