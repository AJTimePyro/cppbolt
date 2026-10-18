#include <cstdio>

template <typename K, typename V>
class KeyValue {
public:
    K key;
    V value;
    KeyValue(K k, V v) : key(k), value(v) {}
    void print() {
        std::printf("KV: key=%d, val=%d\n", (int)key, (int)value);
    }
};

int main() {
    KeyValue<int, int> kv1(1, 100);
    KeyValue<char, int> kv2('Z', 200);

    kv1.print();
    kv2.print();
    return 0;
}
