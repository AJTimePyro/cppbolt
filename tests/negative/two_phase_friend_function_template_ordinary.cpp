namespace N { template <class T> struct S {
    template <class U> friend void probe(S, U) {}
}; }
int main() { probe(1); }
