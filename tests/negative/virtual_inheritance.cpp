struct Base {
    int value;
};

struct Derived : public virtual Base {
    int extra;
};

int main() {
    Derived derived;
    derived.value = 1;
    return derived.value;
}
