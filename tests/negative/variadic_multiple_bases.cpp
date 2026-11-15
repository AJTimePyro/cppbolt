struct Base1 { int x; };
struct Base2 { int y; };

template <typename... Bases>
struct MultiDerived : Bases... {
    int z;
};

int main() {
    MultiDerived<Base1, Base2> obj;
    return 0;
}
