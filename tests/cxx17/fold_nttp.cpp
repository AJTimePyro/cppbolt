#include <cstdio>

template <int N>
struct FixedBuffer {
    int size() const {
        return N;
    }
};

template <int... Sizes>
struct BufferGroup {
    FixedBuffer<(Sizes + ...)> buffer;
    FixedBuffer<(100 + ... + Sizes)> buffer_offset;
    int total_size() const {
        return buffer.size();
    }
    int offset_size() const {
        return buffer_offset.size();
    }
};

template <int... Sizes>
struct EmptyBufferGroup {
    FixedBuffer<(Sizes + ... + 42)> buffer;
    int size() const {
        return buffer.size();
    }
};

int main() {
    BufferGroup<4, 8, 16> group;
    printf("total_size: %d\n", group.total_size());
    printf("offset_size: %d\n", group.offset_size());
    EmptyBufferGroup<> empty_group;
    printf("empty_size: %d\n", empty_group.size());
    return 0;
}
