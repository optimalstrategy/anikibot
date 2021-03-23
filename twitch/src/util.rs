use std::{
    convert::{AsRef, From},
    fmt,
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
};
use std::{slice, str};

#[derive(Clone, Copy)]
pub struct UnsafeSlice {
    ptr: *const u8,
    len: usize,
}

impl UnsafeSlice {
    pub fn len(&self) -> usize { self.len }

    pub fn as_str<'a>(&self) -> &'a str {
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(self.ptr, self.len)) }
    }
}

impl From<&str> for UnsafeSlice {
    fn from(value: &str) -> UnsafeSlice {
        UnsafeSlice {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

impl AsRef<str> for UnsafeSlice {
    fn as_ref(&self) -> &str { str::from_utf8(unsafe { slice::from_raw_parts(self.ptr, self.len) }).unwrap() }
}
impl Debug for UnsafeSlice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Debug::fmt(AsRef::<str>::as_ref(self), f) }
}
impl Display for UnsafeSlice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Display::fmt(AsRef::<str>::as_ref(self), f) }
}
impl Eq for UnsafeSlice {}
impl PartialEq<UnsafeSlice> for UnsafeSlice {
    fn eq(&self, other: &UnsafeSlice) -> bool { (AsRef::<str>::as_ref(self)).eq(AsRef::<str>::as_ref(other)) }
}
impl Hash for UnsafeSlice {
    fn hash<H: Hasher>(&self, state: &mut H) { (AsRef::<str>::as_ref(self)).hash(state) }
}

/* #[macro_export]
macro_rules! impl_unsafe_slice_getters {
    ($S:ident, $($field:ident),+) => {
        impl $S {
            $(
                #[inline]
                pub fn $field(&self) -> &str {
                    self.$field.as_ref()
                }
            )+
        }
    };
} */

#[macro_export]
macro_rules! impl_unsafe_slice_getters {
    (opt $field:ident) => {
        #[inline]
        pub fn $field(&self) -> Option<&str> {
            self.$field.as_ref().map(|v| v.as_str())
        }
    };
    (vec $field:ident) => {
        #[inline]
        pub fn $field(&self) -> impl Iterator<Item=&str> + '_ {
            self.$field.iter().map(|v| v.as_str())
        }
    };
    (none $field:ident) => {
        #[inline]
        pub fn $field(&self) -> &str {
            self.$field.as_str()
        }
    };
    ($S:ident, $($prefix:ident $field:ident),+) => {
        impl $S {
            $(
                impl_unsafe_slice_getters!($prefix $field);
            )+
        }
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn hash_map() {
        let data = "Hello".to_string();
        let slice: UnsafeSlice = (&data[..]).into();

        let mut map = HashMap::<UnsafeSlice, UnsafeSlice>::new();
        map.insert(slice, slice);
        assert_eq!(map.get(&slice).unwrap(), &slice);
    }
}
