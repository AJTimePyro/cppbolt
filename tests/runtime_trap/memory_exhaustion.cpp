#include <cstddef>

int main() {
    while (true) {
        (void)new char[64 * 1024 * 1024];
    }
    return 0;
}
