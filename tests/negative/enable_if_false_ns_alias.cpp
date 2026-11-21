#include <type_traits>

namespace N {
using Bad = std::enable_if<false, int>::type;
}

int main() {}
