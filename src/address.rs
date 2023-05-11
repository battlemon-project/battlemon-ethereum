use std::fmt::LowerHex;
use ethers::addressbook::Address;

pub trait ToHex
where
    Self: LowerHex,
{
    fn to_hex(&self) -> String {
        format!("0x{:x}", self)
    }
}

impl ToHex for Address {}
