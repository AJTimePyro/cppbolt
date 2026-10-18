class Abstract {
public:
    virtual void run() = 0;
};

int main() {
    Abstract a; // error: cannot instantiate abstract class
    return 0;
}
