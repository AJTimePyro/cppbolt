#include <iostream>
#include <vector>

void print_vector_const(const std::vector<double>& v) {
    std::cout << "const range-for:";
    for (double x : v) {
        std::cout << " " << x;
    }
    std::cout << std::endl;

    std::cout << "const iterator:";
    for (std::vector<double>::const_iterator it = v.begin(); it != v.end(); ++it) {
        std::cout << " " << *it;
    }
    std::cout << std::endl;
}

int main() {
    std::vector<double> v;
    v.push_back(1.5);
    v.push_back(2.5);
    v.push_back(3.5);

    double sum = 0.0;
    for (double x : v) {
        sum += x;
    }
    std::cout << "sum: " << sum << std::endl;

    print_vector_const(v);
    return 0;
}
