fn pick_winner<'a>(entries: &'a [Entry], rng: &mut impl Rng) -> &'a Entry {
    &entries[rng.gen_range(0..entries.len())]
}

fn main() {
    // one generator, seeded once, handed down to everything that draws
    let mut rng = StdRng::from_entropy();
    Raffle::open(&mut rng).run();
}

#[test]
fn the_draw_is_reproducible() {
    // the same seed, the same winner, every run
    let entries = [Entry::new("ada"), Entry::new("grace")];
    let mut rng = StdRng::seed_from_u64(42);
    assert_eq!(pick_winner(&entries, &mut rng).name(), "grace");
}
