namespace A {
    template<class T> struct Outer {
        template<class U> struct Inner {};
    };
}

using Bad = A::Outer<int>::Inner<double>::type;

int main() {}
