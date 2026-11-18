struct S {
    int plain() { return 1; }
    int ne() noexcept { return 2; }
};
int main() {
    int (S::* pplain)() = &S::plain;
    int (S::* pne)() noexcept = pplain;   // cannot add noexcept
    (void)pne;
    return 0;
}
