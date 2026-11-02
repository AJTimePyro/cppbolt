// The diagnostic must name the built-in header and the line inside it,
// not the file that included it.
#include <map>

int main() {
    std::map<int, int> m;
    return 0;
}
