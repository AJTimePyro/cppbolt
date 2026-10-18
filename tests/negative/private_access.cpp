class Secret {
private:
    int hidden;
public:
    Secret() : hidden(42) {}
};

int main() {
    Secret s;
    s.hidden = 100; // error: private member
    return 0;
}
