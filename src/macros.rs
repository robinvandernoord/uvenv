pub mod iterable_enum_macro {
    /// [Source: Stackoverflow](https://stackoverflow.com/questions/21371534/in-rust-is-there-a-way-to-iterate-through-the-values-of-an-enum).
    macro_rules! iterable_enum {(
  $(#[$derives:meta])*
  $pub:vis enum $name:ident {
    $(
      $(#[$nested_meta:meta])*
      $member:ident,
    )*
  }) => {
    const _MEMBERS_COUNT:usize = iterable_enum!(@count $($member)*);
    $(#[$derives])*
    $pub enum $name {
      $($(#[$nested_meta])* $member),*
    }
    impl $name {
      pub fn into_iter() -> core::array::IntoIter<$name, _MEMBERS_COUNT> {
        [$($name::$member,)*].into_iter()
      }
    }
  };
  (@count) => (0_usize);
  (@count  $x:tt $($xs:tt)* ) => (1_usize + iterable_enum!(@count $($xs)*));
}
    pub(crate) use iterable_enum; // <-- the trick
}
