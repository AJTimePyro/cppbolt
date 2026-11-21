//! Tests for type helpers.

use super::types::Type;

#[test]
fn test_pack_expansion_type() {
    let ty = Type::PackExpansion(Box::new(Type::TypeParam("T".to_string())));
    assert_eq!(ty.to_c_type(), "T...");
    assert_eq!(ty.strip_all_const(), ty);
}
