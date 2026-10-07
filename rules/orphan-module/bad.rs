mod utils {
    pub fn distance_between(from: &GpsCoordinates, to: &GpsCoordinates) -> Distance {
        Distance::haversine(from.latitude, from.longitude, to.latitude, to.longitude)
    }

    pub fn format_coordinates(coordinates: &GpsCoordinates) -> String {
        format!("{}, {}", coordinates.latitude, coordinates.longitude)
    }

    pub fn parse_coordinates(input: &str) -> Result<GpsCoordinates, InvalidCoordinates> {
        let (latitude, longitude) = input.split_once(',').ok_or(InvalidCoordinates::MissingComma)?;
        Ok(GpsCoordinates {
            latitude: latitude.parse()?,
            longitude: longitude.parse()?,
        })
    }
}
