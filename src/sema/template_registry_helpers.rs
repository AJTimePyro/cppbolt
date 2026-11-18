//! Helpers for template registry.

use crate::name::QualifiedName;
use std::collections::HashMap;

pub(crate) fn insert_template_aliases<T: Clone>(
    map: &mut HashMap<String, T>,
    name: String,
    item: T,
) {
    if let Some(short) = name.split("::").last() {
        map.insert(short.to_string(), item.clone());
    }
    let qualified = QualifiedName::new(&name);
    if qualified.is_global() {
        map.insert(qualified.std_name().into_owned(), item.clone());
    } else if qualified.in_std() {
        map.insert(qualified.plain().to_string(), item.clone());
    }
    map.insert(name, item);
}

