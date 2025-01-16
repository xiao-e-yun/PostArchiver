#[macro_export]
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
