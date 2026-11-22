#include <cstdio>

struct HasDtor {
    int val;
    HasDtor(int v = 0) : val(v) {}
    ~HasDtor() { printf("dtor\n"); }
    HasDtor(const HasDtor& o) : val(o.val) { printf("has_dtor copy ctor\n"); }
};

struct HasCopyCtor {
    int val;
    HasCopyCtor(int v = 0) : val(v) {}
    HasCopyCtor(const HasCopyCtor& o) : val(o.val) { printf("has_copy copy ctor\n"); }
};

int main() {
    HasDtor d1{10};
    printf("-- move HasDtor --\n");
    HasDtor d2 = static_cast<HasDtor&&>(d1);

    HasCopyCtor c1{20};
    printf("-- move HasCopyCtor --\n");
    HasCopyCtor c2 = static_cast<HasCopyCtor&&>(c1);
    printf("done\n");
    return 0;
}
