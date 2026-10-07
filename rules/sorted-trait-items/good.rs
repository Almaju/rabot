trait Persist {
    const TABLE: &'static str;
    type Error;
    fn load(id: &Id, store: &Store) -> Result<Self, Self::Error>;
    fn save(&self, store: &Store) -> Result<(), SaveError>;
}
