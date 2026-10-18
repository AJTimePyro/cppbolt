namespace original {
int value = 3;
}

namespace alias = original;

int main() {
    return alias::value;
}
