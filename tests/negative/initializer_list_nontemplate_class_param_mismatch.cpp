#include <vector>

struct X {
    int n;
    X(std::vector<int> v) : n((int)v.size()) {}
};

int main() {
    std::vector<double> d;
    d.push_back(1.0);
    X x(d);
    return x.n;
}