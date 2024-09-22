use serde::{Deserialize, Serialize};

use crate::scopes::ScopeList;

#[derive(Debug, Hash, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Opaque<T>(T);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub scopes: ScopeList,
    pub username: Option<String>,
    opaque: Opaque<UnsharedData>,
}

impl Session {
    pub fn new_empty() -> Self {
        Self {
            scopes: ScopeList::EMPTY,
            username: None,
            opaque: Opaque(UnsharedData {}),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsharedData {}
