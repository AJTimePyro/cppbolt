// Substituting the body of a selected member template is not SFINAE.
struct S {
    template <typename T> int bad(T x) { return x.no_such_member; }
    int bad(...) { return 2; }
};

int main() {
    S s;
    return s.bad(1);
}
