struct Base {
    int id;
};

struct Derived : Base {
    int extra;
};

void use(Derived* target) {
    target->extra = 1;
}

int main() {
    Base base;
    use(&base);
    return 0;
}
