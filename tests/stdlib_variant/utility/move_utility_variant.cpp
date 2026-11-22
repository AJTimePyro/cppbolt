#include <cstdio>
#include <utility>

struct Buffer {
    int size;
    int* ptr;

    Buffer(int s) : size(s), ptr(new int[s]) {
        for (int i = 0; i < s; i++) ptr[i] = i + 1;
    }

    ~Buffer() {
        if (ptr) {
            delete[] ptr;
            ptr = nullptr;
        }
    }

    Buffer(const Buffer& other) : size(other.size), ptr(new int[other.size]) {
        for (int i = 0; i < size; i++) ptr[i] = other.ptr[i];
        printf("copy buf\n");
    }

    Buffer(Buffer&& other) noexcept : size(other.size), ptr(other.ptr) {
        other.size = 0;
        other.ptr = nullptr;
        printf("move buf\n");
    }

    Buffer& operator=(Buffer&& other) noexcept {
        if (this != &other) {
            if (ptr) delete[] ptr;
            size = other.size;
            ptr = other.ptr;
            other.size = 0;
            other.ptr = nullptr;
            printf("move assign buf\n");
        }
        return *this;
    }
};

template <typename T>
T transfer(T&& x) {
    return std::forward<T>(x);
}

int main() {
    Buffer b1(3);
    Buffer b2 = transfer(std::move(b1));
    printf("b1 size: %d, b2 size: %d\n", b1.size, b2.size);

    Buffer b3(2);
    b3 = std::move(b2);
    printf("b2 size: %d, b3 size: %d\n", b2.size, b3.size);
    return 0;
}
