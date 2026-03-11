# airport-data

[![Crates.io](https://img.shields.io/crates/v/airport-data.svg)](https://crates.io/crates/airport-data)
[![Documentation](https://docs.rs/airport-data/badge.svg)](https://docs.rs/airport-data)
[![License: CC BY 4.0](https://img.shields.io/badge/License-CC%20BY%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by/4.0/)

A comprehensive Rust library for retrieving airport information by IATA codes, ICAO codes, and various other criteria. Provides access to a dataset of 5,000+ airports worldwide with detailed information including coordinates, timezone, type, and external links.

All airport data is embedded directly into the compiled binary — no network requests or external files needed at runtime.

**Website**: [airportdata.dev](https://airportdata.dev) | **GitHub**: [aashishvanand/airport-data-rust](https://github.com/aashishvanand/airport-data-rust) | **Docs**: [docs.rs/airport-data](https://docs.rs/airport-data)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
airport-data = "0.1"
```

## Quick Start

```rust
use airport_data::AirportData;

let db = AirportData::new();

// Look up by IATA code
let airport = db.get_airport_by_iata("SIN").unwrap();
assert_eq!(airport.airport, "Singapore Changi Airport");

// Look up by ICAO code
let airport = db.get_airport_by_icao("WSSS").unwrap();
assert_eq!(airport.country_code, "SG");
```

## Features

### Core Search

```rust
use airport_data::AirportData;

let db = AirportData::new();

// Single airport lookup
let airport = db.get_airport_by_iata("JFK").unwrap();
println!("{} ({})", airport.airport, airport.country_code);

// Search by name (case-insensitive substring match)
let results = db.search_by_name("heathrow");
println!("Found {} result(s)", results.len());

// Autocomplete suggestions
let suggestions = db.get_autocomplete_suggestions("sin", 5);
```

### Geographic Queries

```rust
use airport_data::AirportData;

let db = AirportData::new();

// Find airports within 50 km of a point
let nearby = db.find_nearby_airports(1.3644, 103.9915, 50.0);

// Calculate distance between two airports (in km)
let distance = db.calculate_distance("SIN", "KUL").unwrap();

// Find the nearest airport to coordinates
let nearest = db.find_nearest_airport(51.4700, -0.4543).unwrap();
```

### Filtering

```rust
use airport_data::{AirportData, AirportFilter};

let db = AirportData::new();

// By country, continent, type, or timezone
let sg_airports = db.get_airports_by_country_code("SG");
let asia_airports = db.get_airports_by_continent("AS");
let large = db.get_airports_by_type("large_airport");

// Advanced filtering
let filter = AirportFilter {
    country_code: Some("US".to_string()),
    airport_type: Some("large_airport".to_string()),
    has_scheduled_service: Some(true),
    min_runway_ft: Some(10000),
    ..Default::default()
};
let results = db.find_airports(&filter);
```

### Statistics

```rust
use airport_data::AirportData;

let db = AirportData::new();

let stats = db.get_airport_stats_by_country("US").unwrap();
println!("Total airports: {}", stats.total);
println!("With scheduled service: {}", stats.with_scheduled_service);

let continent_stats = db.get_airport_stats_by_continent("EU").unwrap();
```

### Distance Matrix

```rust
use airport_data::AirportData;

let db = AirportData::new();

let codes = vec!["SIN".to_string(), "KUL".to_string(), "BKK".to_string()];
let matrix = db.calculate_distance_matrix(&codes).unwrap();
```

### Validation

```rust
use airport_data::AirportData;

let db = AirportData::new();

assert!(db.validate_iata_code("SIN"));
assert!(!db.validate_iata_code("ZZZ"));

assert!(db.validate_icao_code("WSSS"));
```

## Airport Data Fields

Each `Airport` struct contains:

| Field | Type | Description |
|-------|------|-------------|
| `iata` | `String` | 3-letter IATA code |
| `icao` | `String` | 4-letter ICAO code |
| `airport` | `String` | Airport name |
| `latitude` | `f64` | Latitude |
| `longitude` | `f64` | Longitude |
| `timezone` | `String` | IANA timezone (e.g. "Asia/Singapore") |
| `utc` | `Option<f64>` | UTC offset |
| `country_code` | `String` | 2-letter country code |
| `continent` | `String` | 2-letter continent code |
| `elevation_ft` | `Option<i64>` | Elevation in feet |
| `airport_type` | `String` | Type (large_airport, medium_airport, small_airport, heliport, seaplane_base) |
| `scheduled_service` | `bool` | Has scheduled commercial service |
| `runway_length` | `Option<i64>` | Longest runway in feet |
| `wikipedia` | `String` | Wikipedia URL |
| `website` | `String` | Official website URL |
| `flightradar24_url` | `String` | FlightRadar24 URL |
| `radarbox_url` | `String` | RadarBox URL |
| `flightaware_url` | `String` | FlightAware URL |

## How It Works

The airport dataset (`airports.json`) is gzip-compressed at build time and embedded into the binary via `include_bytes!`. On first access, the data is decompressed and deserialized into memory, then cached for the lifetime of the process. The `AirportData` struct is zero-cost to construct.

## License

This project is licensed under the [Creative Commons Attribution 4.0 International License (CC-BY-4.0)](https://creativecommons.org/licenses/by/4.0/).
