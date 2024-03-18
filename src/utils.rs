use std::borrow::Borrow;

use crate::apis::OptionId;

pub fn is_default<T: Default + PartialEq>(a: &T) -> bool {
    a == &<T as Default>::default()
}
pub fn option_id_is_none(id: impl Borrow<OptionId>) -> bool {
    id.borrow().is_none()
}
