macro_rules! string_id {
    ($(#[$doc:meta])* $name:ident, $field:expr, $validate:ident) => {
        $(#[$doc])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// Validate and canonicalize the identifier; deserialization enforces the same rules.
            pub fn new(value: impl Into<String>) -> Result<Self, $crate::validation::ValidationError> {
                $crate::ids::$validate(value.into(), $field).map(Self)
            }

            /// Borrow the validated canonical wire representation.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume the identifier into its canonical wire representation.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::write!(f, "{}({:?})", core::stringify!($name), self.0)
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl TryFrom<String> for $name {
            type Error = $crate::validation::ValidationError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $crate::validation::ValidationError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

pub(super) use string_id;
