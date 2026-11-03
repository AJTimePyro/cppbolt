#include <string>
#include <vector>

int main() {
    std::vector<std::vector<int> > v{{1, 2}, {std::string("bad")}};
    return 0;
}