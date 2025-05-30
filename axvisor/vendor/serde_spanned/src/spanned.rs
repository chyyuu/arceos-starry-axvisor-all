use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use alloc::string::String;

// Currently serde itself doesn't have a spanned type, so we map our `Spanned`
// to a special value in the serde data model. Namely one with these special
// fields/struct names.
//
// In general, supported deserializers should catch this and not literally emit
// these strings but rather emit `Spanned` as they're intended.
#[doc(hidden)]
#[cfg(feature = "serde")]
pub const NAME: &str = "$__serde_spanned_private_Spanned";
#[doc(hidden)]
#[cfg(feature = "serde")]
pub const START_FIELD: &str = "$__serde_spanned_private_start";
#[doc(hidden)]
#[cfg(feature = "serde")]
pub const END_FIELD: &str = "$__serde_spanned_private_end";
#[doc(hidden)]
#[cfg(feature = "serde")]
pub const VALUE_FIELD: &str = "$__serde_spanned_private_value";
#[doc(hidden)]
#[cfg(feature = "serde")]
pub fn is_spanned(name: &'static str, fields: &'static [&'static str]) -> bool {
    name == NAME && fields == [START_FIELD, END_FIELD, VALUE_FIELD]
}

/// A spanned value, indicating the range at which it is defined in the source.
#[derive(Clone, Debug)]
pub struct Spanned<T> {
    /// Byte range
    span: core::ops::Range<usize>,
    /// The spanned value.
    value: T,
}

impl<T> Spanned<T> {
    /// Create a spanned value encompassing the given byte range.
    ///
    /// # Example
    ///
    /// Transposing a `Spanned<Enum<T>>` into `Enum<Spanned<T>>`:
    ///
    /// ```
    /// use serde::de::{Deserialize, Deserializer};
    /// use serde_untagged::UntaggedEnumVisitor;
    /// use toml::Spanned;
    ///
    /// pub enum Dependency {
    ///     Simple(Spanned<String>),
    ///     Detailed(Spanned<DetailedDependency>),
    /// }
    ///
    /// impl<'de> Deserialize<'de> for Dependency {
    ///     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Deserializer<'de>,
    ///     {
    ///         enum DependencyKind {
    ///             Simple(String),
    ///             Detailed(DetailedDependency),
    ///         }
    ///
    ///         impl<'de> Deserialize<'de> for DependencyKind {
    ///             fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    ///             where
    ///                 D: Deserializer<'de>,
    ///             {
    ///                 UntaggedEnumVisitor::new()
    ///                     .expecting(
    ///                         "a version string like \"0.9.8\" or a \
    ///                             detailed dependency like { version = \"0.9.8\" }",
    ///                     )
    ///                     .string(|value| Ok(DependencyKind::Simple(value.to_owned())))
    ///                     .map(|value| value.deserialize().map(DependencyKind::Detailed))
    ///                     .deserialize(deserializer)
    ///             }
    ///         }
    ///
    ///         let spanned: Spanned<DependencyKind> = Deserialize::deserialize(deserializer)?;
    ///         let range = spanned.span();
    ///         Ok(match spanned.into_inner() {
    ///             DependencyKind::Simple(simple) => Dependency::Simple(Spanned::new(range, simple)),
    ///             DependencyKind::Detailed(detailed) => Dependency::Detailed(Spanned::new(range, detailed)),
    ///         })
    ///     }
    /// }
    /// #
    /// # type DetailedDependency = alloc::collections::BTreeMap<String, String>;
    /// ```
    pub fn new(range: core::ops::Range<usize>, value: T) -> Self {
        Spanned { span: range, value }
    }

    /// Byte range
    pub fn span(&self) -> core::ops::Range<usize> {
        self.span.clone()
    }

    /// Consumes the spanned value and returns the contained value.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Returns a reference to the contained value.
    pub fn get_ref(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the contained value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl alloc::borrow::Borrow<str> for Spanned<String> {
    fn borrow(&self) -> &str {
        self.get_ref()
    }
}

impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        self.get_ref()
    }
}

impl<T> AsMut<T> for Spanned<T> {
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T: Eq> Eq for Spanned<T> {}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: PartialOrd> PartialOrd for Spanned<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord> Ord for Spanned<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::de::Deserialize<'de> for Spanned<T>
where
    T: serde::de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Spanned<T>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct SpannedVisitor<T>(::core::marker::PhantomData<T>);

        impl<'de, T> serde::de::Visitor<'de> for SpannedVisitor<T>
        where
            T: serde::de::Deserialize<'de>,
        {
            type Value = Spanned<T>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("a spanned value")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Spanned<T>, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                if visitor.next_key()? != Some(START_FIELD) {
                    return Err(serde::de::Error::custom("spanned start key not found"));
                }
                let start: usize = visitor.next_value()?;

                if visitor.next_key()? != Some(END_FIELD) {
                    return Err(serde::de::Error::custom("spanned end key not found"));
                }
                let end: usize = visitor.next_value()?;

                if visitor.next_key()? != Some(VALUE_FIELD) {
                    return Err(serde::de::Error::custom("spanned value key not found"));
                }
                let value: T = visitor.next_value()?;

                Ok(Spanned {
                    span: start..end,
                    value,
                })
            }
        }

        static FIELDS: [&str; 3] = [START_FIELD, END_FIELD, VALUE_FIELD];

        let visitor = SpannedVisitor(::core::marker::PhantomData);

        deserializer.deserialize_struct(NAME, &FIELDS, visitor)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::ser::Serialize> serde::ser::Serialize for Spanned<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.value.serialize(serializer)
    }
}
