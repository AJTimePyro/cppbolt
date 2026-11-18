struct S {
    int mut() { return 1; }
};
int main() {
    S s{};
    const S* p = &s;
    int (S::* pm)() = &S::mut;
    return (p->*pm)();
}
