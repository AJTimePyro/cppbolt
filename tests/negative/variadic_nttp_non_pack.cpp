template<int N>
struct IntConst {
    static const int value = N;
};

template<typename T>
struct Bad {
    IntConst<sizeof...(T)> x;
};

int main() {
    Bad<int> b;
    return 0;
}
