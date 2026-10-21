namespace first {
int shared = 1;
}

namespace second {
int shared = 2;
}

using namespace first;
using namespace second;

int main() {
    return shared;
}
