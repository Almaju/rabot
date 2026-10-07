impl Cache {
    fn evict(&self, key: &Key) -> Result<(), EvictError> {
        if let Err(error) = self.invalidate(key) {
            warn!(%key, %error, "cache entry survives invalidation; serving stale until TTL");
        }
        std::fs::remove_file(self.path_for(key))?;
        Ok(())
    }
}
