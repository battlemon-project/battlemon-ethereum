use ethers::addressbook::Address;
use std::fmt::LowerHex;

pub trait ToHex
where
    Self: LowerHex,
{
    fn to_hex(&self) -> String {
        format!("0x{:x}", self).to_lowercase()
    }
}

impl ToHex for Address {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_hex_works() {
        let expected = "0x4675c7e5baafbffbca748158becba61ef3b0a263";
        let actual: Address = expected.parse().unwrap();

        assert_eq!(expected, actual.to_hex());
    }
}
