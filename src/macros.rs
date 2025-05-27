/// This macro is wrraper tuple struct  
///
/// It will implement the following traits for the tuple struct:
/// - `Deref`
/// - `DerefMut`
/// - `From<TargetType> for RawType` (for the inner type)
/// - `From<RawType> for TargetType` (for the tuple struct)
///
/// # Examples
/// ```ignore
/// #[derive(Debug ,PartialEq, Eq)]
/// struct Id(u32);
/// wrraper!(Id: u32);
///
/// let id: Id = 1.into();
/// assert_eq!(id, Id(1));
///
/// let raw: u32 = id.into();
/// assert_eq!(raw, 1);
/// ```
macro_rules! wrraper {
    ($($t:ty: $f:ty),*) => {
        $(
            impl core::ops::Deref for $t {
                type Target = $f;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl core::ops::DerefMut for $t {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }

            impl From<$f> for $t {
                fn from(f: $f) -> Self {
                    Self(f)
                }
            }

            impl From<$t> for $f {
                fn from(t: $t) -> Self {
                    t.0
                }
            }
        )*
    };
}

pub(crate) use wrraper;
