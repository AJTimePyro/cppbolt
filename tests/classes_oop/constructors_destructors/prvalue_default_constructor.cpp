#include <stdio.h>

class Value {
public:
    int get() && {
        return 20;
    }
};

struct Empty {};

struct PrimitiveHolder {
    int x;
    int y;
};

int main() {
    int v = Value().get();
    printf("Value: %d\n", v);

    Empty e = Empty();
    printf("Empty constructed\n");

    PrimitiveHolder p = PrimitiveHolder();
    printf("PrimitiveHolder constructed\n");
    return 0;
}
