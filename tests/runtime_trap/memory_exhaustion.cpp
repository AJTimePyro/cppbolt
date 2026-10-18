#include <cstdlib>

int main() {
    while (true) {
        malloc(64 * 1024 * 1024);
    }
    return 0;
}
