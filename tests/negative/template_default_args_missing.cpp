template <typename T, typename U = int>
struct PairDef {
    T a;
    U b;
};

int main() {
    PairDef<> p;
    return 0;
}
