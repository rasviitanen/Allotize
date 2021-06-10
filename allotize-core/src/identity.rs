use std::fmt;

#[derive(Clone)]
pub struct Identity {
    pub username: String,
}

impl fmt::Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.username)
    }
}

impl Identity {
    /// Creates a new identity from a username
    pub fn new(name: &str) -> Identity {
        Identity {
            username: name.into(),
        }
    }
}

impl AsRef<Identity> for Identity {
    fn as_ref(&self) -> &Identity {
        self
    }
}
