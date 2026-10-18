#include <cstdio>

class Transaction {
    int tx_id;
public:
    Transaction(int id) : tx_id(id) {
        std::printf("BEGIN tx %d\n", tx_id);
    }
    ~Transaction() {
        std::printf("ROLLBACK/END tx %d\n", tx_id);
    }
};

int execute_tx(int code) {
    Transaction top(100);
    if (code == 0) {
        Transaction nested(101);
        return -99;
    }
    Transaction comm(102);
    return code * 3;
}

int main() {
    std::printf("Run 1:\n");
    int ans1 = execute_tx(0);
    std::printf("ans1: %d\n", ans1);

    std::printf("Run 2:\n");
    int ans2 = execute_tx(7);
    std::printf("ans2: %d\n", ans2);
    return 0;
}
