struct Base {
    typedef int value_type;
};

struct Derived : Base {
    int value_type;
};

template<class T>
int f(typename T::value_type*) {
    return 1;
}

int main() {
    return f<Derived>(0);
}
