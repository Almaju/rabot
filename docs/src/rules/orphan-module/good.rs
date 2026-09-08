struct GpsCoordinates {
    latitude: Latitude,
    longitude: Longitude,
}

impl GpsCoordinates {
    fn parse(input: &str) -> Result<Self, InvalidCoordinates> {
        let (latitude, longitude) = input.split_once(',').ok_or(InvalidCoordinates::MissingComma)?;
        Ok(Self {
            latitude: latitude.parse()?,
            longitude: longitude.parse()?,
        })
    }

    fn display(&self) -> String {
        format!("{}, {}", self.latitude, self.longitude)
    }

    fn distance_to(&self, other: &Self) -> Distance {
        Distance::haversine(self, other)
    }
}
