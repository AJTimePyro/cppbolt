#include <iostream>

namespace Outer {
    namespace Inner {
        struct Box { int w; int h; };
        int calculate(Box b) { return b.w * b.h; }
        int secret = 100;
    }
    namespace NestedAlias = Inner;
}

namespace Alias1 = Outer::Inner;
namespace Alias2 = Alias1;

int main() {
    Alias1::Box b{3, 4};
    std::cout << "area: " << Alias1::calculate(b) << std::endl;
    std::cout << "secret: " << Alias2::secret << std::endl;
    std::cout << "nested_alias: " << Outer::NestedAlias::secret << std::endl;

    {
        namespace LocalAlias = Outer::Inner;
        std::cout << "local: " << LocalAlias::secret << std::endl;
    }

    using namespace Alias2;
    std::cout << "using_alias: " << secret << std::endl;

    return 0;
}
