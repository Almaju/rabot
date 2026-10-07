trait Persist {
    fn save(&self, store: &Store) -> Result<(), SaveError>;
    type Error;
    fn load(id: &Id, store: &Store) -> Result<Self, Self::Error>;
    const TABLE: &'static str;
}
