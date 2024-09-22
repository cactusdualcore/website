use serde::{Deserialize, Serialize};

macro_rules! enum_impls {
    (
        $(#[$($meta:meta),*])*
        $vis:vis enum $name:ident {
            $($variant:ident $(= $v:expr)?),*$(,)?
        }
    ) => {
        $(#[$($meta),*])*
        $vis enum $name {
            $($variant $(= $v)?),*
        }

        impl $name {
            pub const fn variants() -> &'static [Self] {
                &[
                    Self::$($variant),*
                ]
            }
        }
    };
}

enum_impls! {
    #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
    #[repr(u32)]
    pub enum Scope {
        Admin = 0,
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ScopeList(pub u32);

impl From<ScopeList> for u32 {
    fn from(scopes: ScopeList) -> Self {
        scopes.0
    }
}

impl ScopeList {
    pub const EMPTY: Self = ScopeList(0);
    pub const ADMIN: Self = Self::from_scope(Scope::Admin);

    const fn from_scope(scope: Scope) -> Self {
        ScopeList(1 << scope as u32)
    }

    pub const fn contains(&self, scope: Scope) -> bool {
        self.0 & (1 << scope as u32) != 0
    }
}

impl From<Scope> for ScopeList {
    #[inline]
    fn from(scope: Scope) -> Self {
        Self::from_scope(scope)
    }
}

impl<'it> FromIterator<&'it Scope> for ScopeList {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'it Scope>>(iter: T) -> Self {
        iter.into_iter().map(|it| *it).collect()
    }
}

impl FromIterator<Scope> for ScopeList {
    fn from_iter<T: IntoIterator<Item = Scope>>(iter: T) -> Self {
        iter.into_iter()
            .map(ScopeList::from)
            .reduce(|_union, scope| _union | scope)
            .unwrap_or(ScopeList::EMPTY)
    }
}

impl std::ops::BitOr for ScopeList {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ScopeList {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitOrAssign<Scope> for ScopeList {
    fn bitor_assign(&mut self, rhs: Scope) {
        self.0 |= 1 << rhs as u32
    }
}

impl From<ScopeList> for Vec<Scope> {
    fn from(scopes: ScopeList) -> Self {
        Scope::variants()
            .iter()
            .map(|&scope| (scope, scope as u32))
            .filter(|(_, idx)| (scopes.0 & (1 << idx)) != 0)
            .map(|(scope, _)| scope)
            .collect()
    }
}
