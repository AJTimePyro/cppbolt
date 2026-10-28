#include <iostream>

int base_counter = 500;

namespace Root {
    int delta = 15;
    namespace SubLevel {
        int delta = 35;
        int evaluate() {
            return ::base_counter + Root::delta + delta;
        }
    }
}

namespace Root {
    int extra_factor = 99;
}

namespace {
    int hidden_flag = 888;
}

namespace ModA {
    int token = 333;
}

namespace ModB {
    int token = 444;
}

namespace ScopeShadow {
    int level = 8;
    int inspect() {
        int level = 800;
        return level + ScopeShadow::level;
    }
}

namespace ProtocolADL {
    struct Packet {
        int payload_len;
    };
    std::ostream& operator<<(std::ostream& os, const Packet& p) {
        os << "Packet[" << p.payload_len << "]";
        return os;
    }
    int triple_payload(const Packet& p) {
        return p.payload_len * 3;
    }
}

namespace CustomMath {
    int reverse() { return 101; }
    int count() { return 202; }
    int max() { return 303; }
}

int main() {
    std::cout << "eval_nested: " << Root::SubLevel::evaluate() << std::endl;
    std::cout << "extra_root: " << Root::extra_factor << std::endl;
    std::cout << "hidden: " << hidden_flag << std::endl;
    std::cout << "modules: " << ModA::token << " vs " << ModB::token << std::endl;
    std::cout << "shadow_result: " << ScopeShadow::inspect() << std::endl;

    ProtocolADL::Packet pkt;
    pkt.payload_len = 12;
    std::cout << "adl_out: " << pkt << std::endl;
    std::cout << "adl_fn: " << triple_payload(pkt) << std::endl;

    using namespace std;
    using namespace CustomMath;
    cout << "custom_math: " << reverse() << " " << count() << " " << max() << endl;

    using ModB::token;
    cout << "using_mod: " << token << endl;

    return 0;
}
