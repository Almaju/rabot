impl Cache {
    fn evict(&self, key: &Key) {
        match self.invalidate(key) {
            Ok(()) => {}
            Err(_) => {} // shouldn't happen
        }
        std::fs::remove_file(self.path_for(key)).ok();
    }
}
