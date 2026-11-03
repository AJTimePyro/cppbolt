#include <iostream>

template <typename K, typename V>
struct KeyValueStore {
    struct Entry {
        K key;
        V value;
        Entry(K k, V v) : key(k), value(v) {}
    };

    struct Slot {
        Entry item;
        bool occupied;
        Slot() : item(K(), V()), occupied(false) {}
        Slot(K k, V v) : item(k, v), occupied(true) {}
    };

    typedef typename KeyValueStore<K, V>::Slot SlotType;

    Slot slots[4];
    int count;

    KeyValueStore() : count(0) {}

    void insert(K k, V v) {
        if (count < 4) {
            slots[count] = Slot(k, v);
            count++;
        }
    }

    void display() const {
        for (int i = 0; i < count; ++i) {
            std::cout << slots[i].item.key << ": " << slots[i].item.value << "\n";
        }
    }
};

int main() {
    KeyValueStore<int, int> store;
    store.insert(1, 100);
    store.insert(2, 200);
    store.display();

    typename KeyValueStore<int, int>::Entry single(42, 999);
    std::cout << "single: " << single.key << " = " << single.value << "\n";

    return 0;
}
