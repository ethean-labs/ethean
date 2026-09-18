//! Identity smoke checks for Phase 01.

#[test]
fn package_name_is_ethean() {
    assert_eq!(env!("CARGO_PKG_NAME"), "ethean");
}

#[test]
fn package_name_is_not_panro() {
    assert_ne!(env!("CARGO_PKG_NAME"), "panro");
}

#[test]
fn public_client_type_exists() {
    let name = std::any::type_name::<ethean::EtheanClient>();
    assert!(name.contains("EtheanClient"), "unexpected type name: {name}");
}
