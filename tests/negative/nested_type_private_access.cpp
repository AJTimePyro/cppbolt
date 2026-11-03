class Outer {
    struct Secret {
        int x;
    };
public:
    int dummy;
};

int main() {
    Outer::Secret s;
    return 0;
}
