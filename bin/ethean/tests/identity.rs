//! Binary package identity checks.

#[test]
fn package_name_is_ethean() {
    assert_eq!(env!("CARGO_PKG_NAME"), "ethean");
}

#[test]
fn package_name_is_not_panro() {
    assert_ne!(env!("CARGO_PKG_NAME"), "panro");
}
