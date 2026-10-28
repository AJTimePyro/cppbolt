#include <cstdio>

class Device {
public:
    virtual void ping() { std::printf("Device ping\n"); }
};

class NetworkNode : public Device {
public:
    virtual void ping() { std::printf("NetworkNode ping\n"); }
};

class Router : public NetworkNode {
public:
    virtual void ping() { std::printf("Router packet route\n"); }
};

void send_ping(Device* d) {
    d->ping();
}

int main() {
    Device dev;
    NetworkNode node;
    Router rtr;

    send_ping(&dev);
    send_ping(&node);
    send_ping(&rtr);
    return 0;
}
