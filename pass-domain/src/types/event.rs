#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventId(pub(crate) String);
display_for_basic!(EventId);

impl EventId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
