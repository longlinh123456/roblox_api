pub mod economy;
pub mod groups;
pub mod users;

type StrPairArray<'a, const N: usize> = [(&'a str, &'a str); N];
