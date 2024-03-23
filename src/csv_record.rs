use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize)]
pub struct CsvRecord {
    coordinates_lat: f32,
    coordinates_long: f32,
    titel: String,
    description: String,
    year: String,
    color: String,
    tag: String,
    link: String,
}
impl std::fmt::Display for CsvRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Coordinates: {}, {},
titel: {},
description: {},
year: {},
color: {},
tag: {},
link: {}",
            self.coordinates_long,
            self.coordinates_lat,
            self.titel,
            self.description,
            self.year,
            self.color,
            self.tag,
            self.link
        )
    }
}
impl CsvRecord {
    pub fn into_dto(self) -> CsvDto {
        CsvDto {
            coordinates: vec![self.coordinates_long, self.coordinates_lat],
            titel: self.titel,
            description: self.description,
            year: self.year,
            color: self.color,
            tag: self.tag,
            link: self.link,
        }
    }
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CsvDto {
    coordinates: Vec<f32>,
    titel: String,
    description: String,
    year: String,
    color: String,
    tag: String,
    link: String,
}
