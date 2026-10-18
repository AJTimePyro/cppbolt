#include <iostream>
#include <string>

int main() {
    int val = 0;
    std::cin >> val;
    std::cout << "val_extracted: " << val << " state_eof: " << std::cin.eof() << " state_fail: " << std::cin.fail() << std::endl;

    int next_ch = std::cin.peek();
    std::cout << "next_ch: " << (char)next_ch << " state_eof: " << std::cin.eof() << " state_fail: " << std::cin.fail() << std::endl;

    char target = '#';
    std::cin.get(target);
    std::cout << "target: " << target << " gcount_read: " << std::cin.gcount() << " state_eof: " << std::cin.eof() << " state_fail: " << std::cin.fail() << std::endl;

    std::string remaining;
    std::getline(std::cin, remaining);
    std::cout << "rem1: (" << remaining << ") state_eof: " << std::cin.eof() << " state_fail: " << std::cin.fail() << std::endl;

    std::string beyond;
    std::getline(std::cin, beyond);
    std::cout << "rem2: (" << beyond << ") state_eof: " << std::cin.eof() << " state_fail: " << std::cin.fail() << std::endl;

    std::cin.clear();
    int read_result = std::cin.get();
    std::cout << "res_after_clear: " << read_result << " state_eof: " << std::cin.eof() << " state_fail: " << std::cin.fail() << std::endl;

    return 0;
}
