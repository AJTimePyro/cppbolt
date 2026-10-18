#include <iostream>
#include <vector>
#include <string>
#include <algorithm>

struct Student {
    std::string name;
    int score;
    Student(std::string n = "", int s = 0) : name(n), score(s) {}
    bool operator<(const Student& o) const {
        return score < o.score;
    }
};

int main() {
    std::vector<Student> students;
    students.push_back(Student("Charlie", 75));
    students.push_back(Student("Alice", 95));
    students.push_back(Student("Bob", 85));

    std::sort(students.begin(), students.end());

    std::cout << "Rankings:" << std::endl;
    for (size_t i = 0; i < students.size(); ++i) {
        std::cout << i + 1 << ". " << students[i].name << " - " << students[i].score << std::endl;
    }
    return 0;
}
