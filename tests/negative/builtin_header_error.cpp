// The diagnostic must name the built-in header and the line inside it,
// not the file that included it.
#include <vector>

struct Abstract {
    virtual void f() = 0;
};

int main() {
    std::vector<Abstract> v;
    return 0;
}
