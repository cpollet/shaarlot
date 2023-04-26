#[derive(Default, PartialEq, Clone, Debug, Copy, PartialOrd, Eq, Ord, Hash)]
pub enum Eventually<T> {
    #[default]
    None,
    Loading,
    Some(T),
}

impl<T> Eventually<T> {
    #[inline]
    pub fn as_ref(&self) -> Eventually<&T> {
        match self {
            Eventually::None => Eventually::None,
            Eventually::Loading => Eventually::Loading,
            Eventually::Some(t) => Eventually::Some(t),
        }
    }

    #[inline]
    pub fn map<U, F>(self, f: F) -> Eventually<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Eventually::Some(x) => Eventually::Some(f(x)),
            Eventually::Loading => Eventually::Loading,
            Eventually::None => Eventually::None,
        }
    }

    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Eventually::Some(x) => x,
            Eventually::None => Default::default(),
            Eventually::Loading => Default::default(),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Eventually::None)
    }

    pub fn is_some(&self) -> bool {
        matches!(self, Eventually::Some(_))
    }
}
