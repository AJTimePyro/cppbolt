struct S {
    int mut() { return 1; }
};
int main() {
    const S cs{};
    int (S::* pm)() = &S::mut;
    return (cs.*pm)();
}
