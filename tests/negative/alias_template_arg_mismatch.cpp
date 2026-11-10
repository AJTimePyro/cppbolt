template <typename T>
using SingleAlias = T*;

int main() {
    SingleAlias<int, double> x;
    return 0;
}
