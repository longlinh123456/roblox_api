pub fn is_default<T: Default + PartialEq>(a: &T) -> bool {
    a == &<T as Default>::default()
}
