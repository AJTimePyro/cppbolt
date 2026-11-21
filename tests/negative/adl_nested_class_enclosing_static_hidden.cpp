namespace N {
    struct Outer {
        struct Inner {};
        static int helper(Inner) { return 1; }
    };
}

int main() {
    N::Outer::Inner item;
    return helper(item);
}
