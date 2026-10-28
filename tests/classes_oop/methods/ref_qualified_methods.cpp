#include <cstdio>

struct Resource {
    int id;

    int get() & {
        return id * 10;
    }

    int get() && {
        return id * 100;
    }
};

Resource make_resource(int id) {
    Resource r;
    r.id = id;
    return r;
}

Resource& get_ref(Resource& r) {
    return r;
}

int main() {
    Resource r;
    r.id = 5;
    int from_lvalue = r.get();
    int from_rvalue = make_resource(5).get();
    int from_ref_call = get_ref(r).get();
    std::printf("lvalue=%d rvalue=%d ref_call=%d\n", from_lvalue, from_rvalue, from_ref_call);
    return 0;
}
