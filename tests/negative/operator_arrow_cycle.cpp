struct B;

struct A {
    B operator->();
};

struct B {
    A operator->();
};

int main() {
    A a;
    return a->val;
}
