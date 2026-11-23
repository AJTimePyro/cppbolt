use crate::common::run_cpp;

#[test]
fn test_partially_constructed_array_unwinds_completed_elements() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int g_counter = 0;
struct Item {
    int id;
    Item() {
        id = ++g_counter;
        std::printf("C:%d ", id);
        if (id == 3) {
            throw 99;
        }
    }
    ~Item() {
        std::printf("D:%d ", id);
    }
};

void test_arr() {
    Item arr[4];
}

int main() {
    try {
        test_arr();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 C:3 D:2 D:1 Caught:99");
    Ok(())
}

#[test]
fn test_member_array_unwinds_on_constructor_failure() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int g_m_counter = 0;
struct MemberItem {
    int id;
    MemberItem() {
        id = ++g_m_counter;
        std::printf("C_M:%d ", id);
    }
    ~MemberItem() {
        std::printf("D_M:%d ", id);
    }
};

struct ThrowingMember {
    ThrowingMember() {
        std::printf("C_Throw ");
        throw 77;
    }
    ~ThrowingMember() {}
};

struct Container {
    MemberItem arr[3];
    ThrowingMember bad;
    Container() {}
};

int main() {
    try {
        Container c;
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "C_M:1 C_M:2 C_M:3 C_Throw D_M:3 D_M:2 D_M:1 Caught:77"
    );
    Ok(())
}
