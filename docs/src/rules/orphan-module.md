# orphan-module

**Level**: warn · **Article**: [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs)

> Every function in your utils file is a method on a type that doesn't exist
> yet. The type is there. You just haven't named it.

## What it checks

A module named `utils`, `util`, `helpers`, `helper`, `common` or `misc`,
whether declared inline or as `mod utils;`.

## Don't

```rust
// utils.rs
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 { .. }
pub fn format_coordinates(lat: f64, lon: f64) -> String { .. }
pub fn validate_lat_lon(lat: f64, lon: f64) -> bool { .. }
```

It starts with one function that has no obvious home. Then fifteen. Then it
is 800 lines and nobody can say what it is about, because it is not about
anything. It is a drawer.

## Do

```rust
// gps_coordinates.rs
struct GpsCoordinates { latitude: f64, longitude: f64 }

impl GpsCoordinates {
    fn distance_to(&self, other: &Self) -> Distance { .. }
    fn display(&self) -> String { .. }
    fn parse(lat: f64, lon: f64) -> Result<Self, InvalidCoordinates> { .. }
}
```

Five functions that share three parameters are a struct. Name it and the
orphans find their home.

## Options

```toml
[naming]
orphan-modules = ["common", "helper", "helpers", "misc", "util", "utils"]
```

## Silence it

```rust
// rabot: allow(orphan-module) test support only: builders and fixtures for the integration suite
mod helpers;
```
