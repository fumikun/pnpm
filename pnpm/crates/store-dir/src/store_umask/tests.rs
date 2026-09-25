use super::StoreUmask;
use pretty_assertions::assert_eq;

#[test]
fn parses_octal_digits() {
    for (value, mask) in
        [("002", 0o002), ("22", 0o022), ("0077", 0o077), ("0o027", 0o027), ("0", 0)]
    {
        assert_eq!(value.parse::<StoreUmask>().unwrap(), StoreUmask(mask), "{value}");
    }
}

#[test]
fn rejects_values_that_are_not_an_octal_umask() {
    for value in ["", "0o", "8", "028", "1000", "-2", "0x12", "abc", " 022"] {
        assert!(value.parse::<StoreUmask>().is_err(), "must reject {value:?}");
    }
}

#[test]
fn rejects_a_mask_that_clears_the_owner_bits() {
    for value in ["100", "400", "700", "777"] {
        assert!(value.parse::<StoreUmask>().is_err(), "must reject {value:?}");
    }
}

#[test]
fn clears_the_masked_bits_from_the_file_mode() {
    let umask = StoreUmask(0o027);
    assert_eq!(umask.file_mode(false), 0o640);
    assert_eq!(umask.file_mode(true), 0o750);
}

#[test]
fn displays_three_octal_digits() {
    assert_eq!(StoreUmask(0o002).to_string(), "002");
    assert_eq!(StoreUmask(0o077).to_string(), "077");
}

#[test]
fn round_trips_through_serde_as_a_string() {
    let json = serde_json::to_value(StoreUmask(0o002)).unwrap();
    assert_eq!(json, serde_json::json!("002"));
    assert_eq!(serde_json::from_value::<StoreUmask>(json).unwrap(), StoreUmask(0o002));
}
