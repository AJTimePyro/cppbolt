#include <cstdio>

class ReadOnlyData {
    int value;
public:
    ReadOnlyData(int v) : value(v) {}
    int get_value() const {
        return value;
    }
};

void inspect(const ReadOnlyData& d) {
    std::printf("inspected: %d\n", d.get_value());
}

int main() {
    ReadOnlyData obj(777);
    inspect(obj);
    return 0;
}
