fn pick_winner(entries: &[Entry]) -> &Entry {
    &entries[rand::thread_rng().gen_range(0..entries.len())]
}
