#include <cstdio>

class DeviceNode {
public:
    static int seq;
    int serial;
    DeviceNode() {
        seq += 10;
        serial = seq;
        std::printf("DeviceNode ctor serial=%d\n", serial);
    }
    ~DeviceNode() {
        std::printf("DeviceNode dtor serial=%d\n", serial);
    }
};

int DeviceNode::seq = 0;

int main() {
    std::printf("allocating device array\n");
    DeviceNode* nodes = new DeviceNode[4];
    std::printf("array ready\n");
    for (int i = 0; i < 4; ++i) {
        std::printf("node %d serial=%d\n", i, nodes[i].serial);
    }
    std::printf("deleting device array\n");
    delete[] nodes;
    std::printf("deletion complete\n");
    return 0;
}
