struct Outer {
    struct Inner {
    private:
        int secret;
    public:
        Inner() : secret(10) {}
    };

    int read_secret(Inner in) {
        return in.secret;
    }
};

int main() {
    Outer o;
    Outer::Inner in;
    return o.read_secret(in);
}
