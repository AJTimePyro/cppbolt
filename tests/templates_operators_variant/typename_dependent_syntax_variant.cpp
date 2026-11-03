#include <cstdio>

template <typename Container>
struct Inspector {
    using elem_type = typename Container::element_type;

    elem_type sum(const typename Container::element_type& a, const typename Container::element_type& b) const {
        return a + b;
    }
};

struct DataBucket {
    using element_type = int;
};

int main() {
    Inspector<DataBucket> insp;
    DataBucket::element_type x = 10;
    DataBucket::element_type y = 25;
    std::printf("Sum: %d\n", insp.sum(x, y));
    return 0;
}
