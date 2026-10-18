#include <iostream>

int global_id = 100;

namespace Alpha {
    int val = 10;
    namespace Beta {
        int val = 20;
        int compute() {
            return val + Alpha::val + ::global_id;
        }
    }
}

namespace Alpha {
    int reopened_val = 55;
}

namespace {
    int internal_secret = 777;
}

namespace ScopeOne {
    int datum = 111;
}

namespace ScopeTwo {
    int datum = 222;
}

namespace HidingTest {
    int shadow = 5;
    int resolve() {
        int shadow = 50;
        return shadow + HidingTest::shadow;
    }
}

namespace MyADL {
    struct Item {
        int count;
    };
    std::ostream& operator<<(std::ostream& os, const Item& item) {
        os << "Item(" << item.count << ")";
        return os;
    }
    int double_item(const Item& item) {
        return item.count * 2;
    }
}

namespace UserAlgs {
    int reverse() { return 10; }
    int count() { return 20; }
    int max() { return 30; }
}

int main() {
    std::cout << "nested: " << Alpha::Beta::compute() << std::endl;
    std::cout << "reopened: " << Alpha::reopened_val << std::endl;
    std::cout << "anon: " << internal_secret << std::endl;
    std::cout << "distinct: " << ScopeOne::datum << " and " << ScopeTwo::datum << std::endl;
    std::cout << "hiding: " << HidingTest::resolve() << std::endl;

    MyADL::Item it;
    it.count = 21;
    std::cout << "adl_stream: " << it << std::endl;
    std::cout << "adl_call: " << double_item(it) << std::endl;

    using namespace std;
    using namespace UserAlgs;
    cout << "user_algs: " << reverse() << " " << count() << " " << max() << endl;

    using ScopeOne::datum;
    cout << "using_decl: " << datum << endl;

    return 0;
}
