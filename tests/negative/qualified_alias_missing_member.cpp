namespace A {
    template<class T> struct Disable {};
}

namespace B {
    using Invalid = A::Disable<int>::type;
}

int main() {}
