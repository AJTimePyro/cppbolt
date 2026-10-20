#include <cstddef>

int main() {
    (void)new char[300 * 1024 * 1024];
    return 0;
}
