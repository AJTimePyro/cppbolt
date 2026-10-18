class AbstractValue {
public:
    virtual int read(int x) = 0;
    virtual ~AbstractValue() {}
};

int main() {
    AbstractValue* p = (AbstractValue*)0;
    return p->read(7);
}
