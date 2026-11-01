namespace original {
int value = 3;
}

namespace other {
int value = 4;
}

namespace alias = original;
namespace alias = other;

int main() {
    return alias::value;
}
