#include <cstdio>

// Held-out variant of global static objects
class DeviceInit {
public:
    int dev_id;
    DeviceInit() : dev_id(10) {
        printf("DeviceInit online\n");
    }
    ~DeviceInit() {
        printf("DeviceInit offline\n");
    }
};

class SubsystemInit {
public:
    int sub_id;
    SubsystemInit() : sub_id(20) {
        printf("SubsystemInit online\n");
    }
    ~SubsystemInit() {
        printf("SubsystemInit offline\n");
    }
};

DeviceInit g_dev;
SubsystemInit g_sub;
int g_buffer[20000];

int buffer_value(int index) { return g_buffer[index]; }

int main() {
    printf("Main running\n");
    printf("buf[0]=%d, buf[10000]=%d, buf[19999]=%d\n",
           buffer_value(0), buffer_value(10000), buffer_value(19999));
    g_buffer[10000] = 888;
    printf("updated buf[10000]=%d\n", buffer_value(10000));
    printf("Main done\n");
    return 0;
}