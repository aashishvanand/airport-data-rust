//! # Airport Data
//!
//! A comprehensive Rust library for retrieving airport information by IATA codes,
//! ICAO codes, and various other criteria. Provides access to a large dataset of
//! airports worldwide with detailed information including coordinates, timezone,
//! type, and external links.
//!
//! ## Quick Start
//!
//! ```rust
//! use airport_data::AirportData;
//!
//! let db = AirportData::new();
//!
//! // Get airport by IATA code
//! let airport = db.get_airport_by_iata("SIN").unwrap();
//! assert_eq!(airport.airport, "Singapore Changi Airport");
//!
//! // Get airport by ICAO code
//! let airport = db.get_airport_by_icao("WSSS").unwrap();
//! assert_eq!(airport.country_code, "SG");
//! ```

use flate2::read::GzDecoder;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::io::Read;

// Embed the compressed data at compile time
static COMPRESSED_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/airports.json.gz"));

static AIRPORTS: Lazy<Vec<Airport>> = Lazy::new(|| {
    let mut decoder = GzDecoder::new(COMPRESSED_DATA);
    let mut json_str = String::new();
    decoder
        .read_to_string(&mut json_str)
        .expect("Failed to decompress airport data");
    serde_json::from_str(&json_str).expect("Failed to parse airport data")
});

// ---------------------------------------------------------------------------
// Custom deserializer helpers
// ---------------------------------------------------------------------------

/// Deserializes a value that can be an integer, float, or an empty string into Option<i64>.
fn deserialize_opt_i64<'de, D>(deserializer: D) -> std::result::Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        Int(i64),
        Float(f64),
        Str(String),
    }
    match NumOrStr::deserialize(deserializer)? {
        NumOrStr::Int(n) => Ok(Some(n)),
        NumOrStr::Float(f) => Ok(Some(f as i64)),
        NumOrStr::Str(s) if s.is_empty() => Ok(None),
        NumOrStr::Str(s) => s.parse::<i64>().ok().map_or(Ok(None), |n| Ok(Some(n))),
    }
}

/// Deserializes a value that can be an integer, float, or an empty string into Option<f64>.
fn deserialize_opt_f64<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        Int(i64),
        Float(f64),
        Str(String),
    }
    match NumOrStr::deserialize(deserializer)? {
        NumOrStr::Int(n) => Ok(Some(n as f64)),
        NumOrStr::Float(f) => Ok(Some(f)),
        NumOrStr::Str(s) if s.is_empty() => Ok(None),
        NumOrStr::Str(s) => s.parse::<f64>().ok().map_or(Ok(None), |n| Ok(Some(n))),
    }
}

/// Deserializes a value that can be a string or an integer into a String.
fn deserialize_string_or_int<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        Str(String),
        Int(i64),
    }
    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::Str(s) => Ok(s),
        StringOrInt::Int(n) => Ok(n.to_string()),
    }
}

/// Deserializes "TRUE"/"FALSE" strings into bool.
fn deserialize_bool_from_string<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes"))
}

// ---------------------------------------------------------------------------
// Airport struct
// ---------------------------------------------------------------------------

/// Represents a single airport with all its associated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airport {
    /// 3-letter IATA code (may be empty for some airports)
    pub iata: String,
    /// 4-letter ICAO code (may be empty for some airports)
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub icao: String,
    /// Timezone identifier (e.g. "Asia/Singapore")
    #[serde(rename = "time")]
    pub timezone: String,
    /// UTC offset (can be fractional, e.g. 5.5 for India; None if unavailable)
    #[serde(deserialize_with = "deserialize_opt_f64")]
    pub utc: Option<f64>,
    /// 2-letter country code
    pub country_code: String,
    /// 2-letter continent code (AS, EU, NA, SA, AF, OC, AN)
    pub continent: String,
    /// Airport name
    pub airport: String,
    /// Latitude coordinate
    pub latitude: f64,
    /// Longitude coordinate
    pub longitude: f64,
    /// Elevation in feet (may be None if data unavailable)
    #[serde(deserialize_with = "deserialize_opt_i64")]
    pub elevation_ft: Option<i64>,
    /// Airport type (large_airport, medium_airport, small_airport, heliport, seaplane_base)
    #[serde(rename = "type")]
    pub airport_type: String,
    /// Whether the airport has scheduled commercial service
    #[serde(deserialize_with = "deserialize_bool_from_string")]
    pub scheduled_service: bool,
    /// Wikipedia URL
    #[serde(default)]
    pub wikipedia: String,
    /// Airport website URL
    #[serde(default)]
    pub website: String,
    /// Longest runway length in feet (may be None if data unavailable)
    #[serde(deserialize_with = "deserialize_opt_i64")]
    pub runway_length: Option<i64>,
    /// FlightRadar24 URL
    #[serde(default)]
    pub flightradar24_url: String,
    /// RadarBox URL
    #[serde(default)]
    pub radarbox_url: String,
    /// FlightAware URL
    #[serde(default)]
    pub flightaware_url: String,
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// External links for an airport.
#[derive(Debug, Clone, Serialize)]
pub struct AirportLinks {
    pub website: Option<String>,
    pub wikipedia: Option<String>,
    pub flightradar24: Option<String>,
    pub radarbox: Option<String>,
    pub flightaware: Option<String>,
}

/// Statistics for airports in a country.
#[derive(Debug, Clone, Serialize)]
pub struct CountryStats {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub with_scheduled_service: usize,
    pub average_runway_length: f64,
    pub average_elevation: f64,
    pub timezones: Vec<String>,
}

/// Statistics for airports on a continent.
#[derive(Debug, Clone, Serialize)]
pub struct ContinentStats {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub by_country: HashMap<String, usize>,
    pub with_scheduled_service: usize,
    pub average_runway_length: f64,
    pub average_elevation: f64,
    pub timezones: Vec<String>,
}

/// Information about an airport in a distance matrix.
#[derive(Debug, Clone, Serialize)]
pub struct AirportInfo {
    pub code: String,
    pub name: String,
    pub iata: String,
    pub icao: String,
}

/// Distance matrix result.
#[derive(Debug, Clone, Serialize)]
pub struct DistanceMatrix {
    pub airports: Vec<AirportInfo>,
    pub distances: HashMap<String, HashMap<String, f64>>,
}

/// An airport with its distance from a reference point.
#[derive(Debug, Clone)]
pub struct NearbyAirport {
    pub airport: Airport,
    /// Distance in kilometers.
    pub distance: f64,
}

/// Filters for advanced airport search.
#[derive(Debug, Clone, Default)]
pub struct AirportFilter {
    pub country_code: Option<String>,
    pub continent: Option<String>,
    pub airport_type: Option<String>,
    pub has_scheduled_service: Option<bool>,
    pub min_runway_ft: Option<i64>,
}

/// Error types for the library.
#[derive(Debug, Clone)]
pub enum AirportError {
    /// No data found for the given code/query.
    NotFound(String),
    /// Invalid input format.
    InvalidInput(String),
}

impl std::fmt::Display for AirportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirportError::NotFound(msg) => write!(f, "{}", msg),
            AirportError::InvalidInput(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AirportError {}

pub type Result<T> = std::result::Result<T, AirportError>;

// ---------------------------------------------------------------------------
// Haversine distance
// ---------------------------------------------------------------------------

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    let d_lat = (lat2 - lat1) * PI / 180.0;
    let d_lon = (lon2 - lon1) * PI / 180.0;
    let lat1_rad = lat1 * PI / 180.0;
    let lat2_rad = lat2 * PI / 180.0;

    let a = (d_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS_KM * c
}

// ---------------------------------------------------------------------------
// Helper: match airport type filter
// ---------------------------------------------------------------------------

fn type_matches(airport_type: &str, filter_type: &str) -> bool {
    let filter_lower = filter_type.to_lowercase();
    if filter_lower == "airport" {
        let at = airport_type.to_lowercase();
        at == "large_airport" || at == "medium_airport" || at == "small_airport"
    } else {
        airport_type.eq_ignore_ascii_case(&filter_lower)
    }
}

/// Helper to apply AirportFilter to an airport.
fn matches_filter(airport: &Airport, filter: &AirportFilter) -> bool {
    if let Some(ref cc) = filter.country_code {
        if !airport.country_code.eq_ignore_ascii_case(cc) {
            return false;
        }
    }
    if let Some(ref cont) = filter.continent {
        if !airport.continent.eq_ignore_ascii_case(cont) {
            return false;
        }
    }
    if let Some(ref t) = filter.airport_type {
        if !type_matches(&airport.airport_type, t) {
            return false;
        }
    }
    if let Some(has_service) = filter.has_scheduled_service {
        if airport.scheduled_service != has_service {
            return false;
        }
    }
    if let Some(min_runway) = filter.min_runway_ft {
        let runway = airport.runway_length.unwrap_or(0);
        if runway < min_runway {
            return false;
        }
    }
    true
}

fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

// ---------------------------------------------------------------------------
// AirportData - main public API
// ---------------------------------------------------------------------------

/// The main entry point for querying airport data.
///
/// All data is loaded lazily on first access and cached for the lifetime of
/// the process. The `AirportData` struct itself is zero-cost to create — it
/// just provides methods to query the shared global data.
pub struct AirportData;

impl AirportData {
    /// Creates a new `AirportData` instance.
    ///
    /// This is cheap — data is loaded lazily on first query.
    pub fn new() -> Self {
        AirportData
    }

    /// Returns a reference to all airports in the database.
    pub fn all_airports(&self) -> &[Airport] {
        &AIRPORTS
    }

    // =======================================================================
    // Core Search Functions
    // =======================================================================

    /// Finds an airport by its 3-letter IATA code.
    ///
    /// Returns the first matching airport, or an error if not found.
    ///
    /// # Example
    /// ```
    /// use airport_data::AirportData;
    /// let db = AirportData::new();
    /// let airport = db.get_airport_by_iata("LHR").unwrap();
    /// assert!(airport.airport.contains("Heathrow"));
    /// ```
    pub fn get_airport_by_iata(&self, iata_code: &str) -> Result<&Airport> {
        let code = iata_code.trim().to_uppercase();
        AIRPORTS
            .iter()
            .find(|a| a.iata.eq_ignore_ascii_case(&code))
            .ok_or_else(|| {
                AirportError::NotFound(format!("No data found for IATA code: {}", iata_code))
            })
    }

    /// Finds all airports matching a 3-letter IATA code.
    ///
    /// Returns a `Vec` of references (usually 0 or 1 results).
    pub fn get_airports_by_iata(&self, iata_code: &str) -> Vec<&Airport> {
        let code = iata_code.trim().to_uppercase();
        AIRPORTS
            .iter()
            .filter(|a| a.iata.eq_ignore_ascii_case(&code))
            .collect()
    }

    /// Finds an airport by its 4-letter ICAO code.
    ///
    /// Returns the first matching airport, or an error if not found.
    pub fn get_airport_by_icao(&self, icao_code: &str) -> Result<&Airport> {
        let code = icao_code.trim().to_uppercase();
        AIRPORTS
            .iter()
            .find(|a| a.icao.eq_ignore_ascii_case(&code))
            .ok_or_else(|| {
                AirportError::NotFound(format!("No data found for ICAO code: {}", icao_code))
            })
    }

    /// Finds all airports matching a 4-letter ICAO code.
    pub fn get_airports_by_icao(&self, icao_code: &str) -> Vec<&Airport> {
        let code = icao_code.trim().to_uppercase();
        AIRPORTS
            .iter()
            .filter(|a| a.icao.eq_ignore_ascii_case(&code))
            .collect()
    }

    /// Searches for airports by name (case-insensitive partial match).
    ///
    /// Query must be at least 2 characters.
    pub fn search_by_name(&self, query: &str) -> Result<Vec<&Airport>> {
        let q = query.trim();
        if q.len() < 2 {
            return Err(AirportError::InvalidInput(
                "Search query must be at least 2 characters".to_string(),
            ));
        }
        let lower = q.to_lowercase();
        let results: Vec<&Airport> = AIRPORTS
            .iter()
            .filter(|a| a.airport.to_lowercase().contains(&lower))
            .collect();
        Ok(results)
    }

    // =======================================================================
    // Geographic Functions
    // =======================================================================

    /// Finds airports within a specified radius (km) of given coordinates.
    ///
    /// Results are sorted by distance (nearest first).
    pub fn find_nearby_airports(&self, lat: f64, lon: f64, radius_km: f64) -> Vec<NearbyAirport> {
        let mut results: Vec<NearbyAirport> = AIRPORTS
            .iter()
            .filter_map(|a| {
                let dist = haversine_distance(lat, lon, a.latitude, a.longitude);
                if dist <= radius_km {
                    Some(NearbyAirport {
                        airport: a.clone(),
                        distance: dist,
                    })
                } else {
                    None
                }
            })
            .collect();
        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Calculates the great-circle distance between two airports in kilometers.
    ///
    /// Accepts either IATA or ICAO codes.
    pub fn calculate_distance(&self, code1: &str, code2: &str) -> Result<f64> {
        let a1 = self.resolve_airport(code1)?;
        let a2 = self.resolve_airport(code2)?;
        Ok(haversine_distance(
            a1.latitude, a1.longitude, a2.latitude, a2.longitude,
        ))
    }

    /// Finds the single nearest airport to given coordinates.
    ///
    /// Optionally applies filters to narrow the search.
    pub fn find_nearest_airport(
        &self,
        lat: f64,
        lon: f64,
        filter: Option<&AirportFilter>,
    ) -> Result<NearbyAirport> {
        let mut best: Option<NearbyAirport> = None;

        for airport in AIRPORTS.iter() {
            if let Some(f) = filter {
                if !matches_filter(airport, f) {
                    continue;
                }
            }
            let dist = haversine_distance(lat, lon, airport.latitude, airport.longitude);
            let is_closer = match &best {
                Some(b) => dist < b.distance,
                None => true,
            };
            if is_closer {
                best = Some(NearbyAirport {
                    airport: airport.clone(),
                    distance: dist,
                });
            }
        }

        best.ok_or_else(|| {
            AirportError::NotFound("No airport found matching the criteria".to_string())
        })
    }

    // =======================================================================
    // Filtering Functions
    // =======================================================================

    /// Finds all airports in a specific country.
    pub fn get_airports_by_country_code(&self, country_code: &str) -> Vec<&Airport> {
        let cc = country_code.trim().to_uppercase();
        AIRPORTS
            .iter()
            .filter(|a| a.country_code.eq_ignore_ascii_case(&cc))
            .collect()
    }

    /// Finds all airports on a specific continent.
    ///
    /// Continent codes: AS, EU, NA, SA, AF, OC, AN
    pub fn get_airports_by_continent(&self, continent_code: &str) -> Vec<&Airport> {
        let cc = continent_code.trim().to_uppercase();
        AIRPORTS
            .iter()
            .filter(|a| a.continent.eq_ignore_ascii_case(&cc))
            .collect()
    }

    /// Finds airports by their type.
    ///
    /// Types: `large_airport`, `medium_airport`, `small_airport`, `heliport`,
    /// `seaplane_base`. Use `"airport"` to match all airport types (large,
    /// medium, small).
    pub fn get_airports_by_type(&self, airport_type: &str) -> Vec<&Airport> {
        AIRPORTS
            .iter()
            .filter(|a| type_matches(&a.airport_type, airport_type))
            .collect()
    }

    /// Finds all airports within a specific timezone.
    pub fn get_airports_by_timezone(&self, timezone: &str) -> Vec<&Airport> {
        AIRPORTS
            .iter()
            .filter(|a| a.timezone == timezone)
            .collect()
    }

    /// Finds airports matching multiple criteria.
    pub fn find_airports(&self, filter: &AirportFilter) -> Vec<&Airport> {
        AIRPORTS
            .iter()
            .filter(|a| matches_filter(a, filter))
            .collect()
    }

    // =======================================================================
    // Autocomplete & Links
    // =======================================================================

    /// Provides autocomplete suggestions for search interfaces.
    ///
    /// Returns up to 10 airports matching the query by name or IATA code.
    /// Query must be at least 2 characters.
    pub fn get_autocomplete_suggestions(&self, query: &str) -> Vec<&Airport> {
        let q = query.trim();
        if q.len() < 2 {
            return Vec::new();
        }
        let lower = q.to_lowercase();
        AIRPORTS
            .iter()
            .filter(|a| {
                a.airport.to_lowercase().contains(&lower)
                    || a.iata.to_lowercase().contains(&lower)
            })
            .take(10)
            .collect()
    }

    /// Gets external links for an airport.
    ///
    /// Accepts either IATA or ICAO code.
    pub fn get_airport_links(&self, code: &str) -> Result<AirportLinks> {
        let airport = self.resolve_airport(code)?;
        Ok(AirportLinks {
            website: non_empty(&airport.website),
            wikipedia: non_empty(&airport.wikipedia),
            flightradar24: non_empty(&airport.flightradar24_url),
            radarbox: non_empty(&airport.radarbox_url),
            flightaware: non_empty(&airport.flightaware_url),
        })
    }

    // =======================================================================
    // Statistical & Analytical Functions
    // =======================================================================

    /// Gets comprehensive statistics about airports in a specific country.
    pub fn get_airport_stats_by_country(&self, country_code: &str) -> Result<CountryStats> {
        let airports = self.get_airports_by_country_code(country_code);
        if airports.is_empty() {
            return Err(AirportError::NotFound(format!(
                "No airports found for country code: {}",
                country_code
            )));
        }

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut with_scheduled_service = 0usize;
        let mut runway_sum = 0f64;
        let mut runway_count = 0usize;
        let mut elevation_sum = 0f64;
        let mut elevation_count = 0usize;
        let mut tz_set: Vec<String> = Vec::new();

        for a in &airports {
            *by_type.entry(a.airport_type.clone()).or_insert(0) += 1;
            if a.scheduled_service {
                with_scheduled_service += 1;
            }
            if let Some(r) = a.runway_length {
                if r > 0 {
                    runway_sum += r as f64;
                    runway_count += 1;
                }
            }
            if let Some(e) = a.elevation_ft {
                elevation_sum += e as f64;
                elevation_count += 1;
            }
            if !tz_set.contains(&a.timezone) {
                tz_set.push(a.timezone.clone());
            }
        }

        tz_set.sort();

        Ok(CountryStats {
            total: airports.len(),
            by_type,
            with_scheduled_service,
            average_runway_length: if runway_count > 0 {
                (runway_sum / runway_count as f64).round()
            } else {
                0.0
            },
            average_elevation: if elevation_count > 0 {
                (elevation_sum / elevation_count as f64).round()
            } else {
                0.0
            },
            timezones: tz_set,
        })
    }

    /// Gets comprehensive statistics about airports on a specific continent.
    pub fn get_airport_stats_by_continent(
        &self,
        continent_code: &str,
    ) -> Result<ContinentStats> {
        let airports = self.get_airports_by_continent(continent_code);
        if airports.is_empty() {
            return Err(AirportError::NotFound(format!(
                "No airports found for continent code: {}",
                continent_code
            )));
        }

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_country: HashMap<String, usize> = HashMap::new();
        let mut with_scheduled_service = 0usize;
        let mut runway_sum = 0f64;
        let mut runway_count = 0usize;
        let mut elevation_sum = 0f64;
        let mut elevation_count = 0usize;
        let mut tz_set: Vec<String> = Vec::new();

        for a in &airports {
            *by_type.entry(a.airport_type.clone()).or_insert(0) += 1;
            *by_country.entry(a.country_code.clone()).or_insert(0) += 1;
            if a.scheduled_service {
                with_scheduled_service += 1;
            }
            if let Some(r) = a.runway_length {
                if r > 0 {
                    runway_sum += r as f64;
                    runway_count += 1;
                }
            }
            if let Some(e) = a.elevation_ft {
                elevation_sum += e as f64;
                elevation_count += 1;
            }
            if !tz_set.contains(&a.timezone) {
                tz_set.push(a.timezone.clone());
            }
        }

        tz_set.sort();

        Ok(ContinentStats {
            total: airports.len(),
            by_type,
            by_country,
            with_scheduled_service,
            average_runway_length: if runway_count > 0 {
                (runway_sum / runway_count as f64).round()
            } else {
                0.0
            },
            average_elevation: if elevation_count > 0 {
                (elevation_sum / elevation_count as f64).round()
            } else {
                0.0
            },
            timezones: tz_set,
        })
    }

    /// Gets the largest airports on a continent sorted by runway length or
    /// elevation.
    ///
    /// `sort_by` can be `"runway"` (default) or `"elevation"`.
    pub fn get_largest_airports_by_continent(
        &self,
        continent_code: &str,
        limit: usize,
        sort_by: &str,
    ) -> Vec<Airport> {
        let mut airports: Vec<Airport> = self
            .get_airports_by_continent(continent_code)
            .into_iter()
            .cloned()
            .collect();

        match sort_by.to_lowercase().as_str() {
            "elevation" => {
                airports.sort_by(|a, b| {
                    let ea = a.elevation_ft.unwrap_or(0);
                    let eb = b.elevation_ft.unwrap_or(0);
                    eb.cmp(&ea)
                });
            }
            _ => {
                // Default: sort by runway length
                airports.sort_by(|a, b| {
                    let ra = a.runway_length.unwrap_or(0);
                    let rb = b.runway_length.unwrap_or(0);
                    rb.cmp(&ra)
                });
            }
        }

        airports.truncate(limit);
        airports
    }

    // =======================================================================
    // Bulk Operations
    // =======================================================================

    /// Fetches multiple airports by their IATA or ICAO codes.
    ///
    /// Returns `None` for codes that are not found.
    pub fn get_multiple_airports(&self, codes: &[&str]) -> Vec<Option<&Airport>> {
        codes
            .iter()
            .map(|code| self.resolve_airport(code).ok())
            .collect()
    }

    /// Calculates distances between all pairs of airports in a list.
    ///
    /// Requires at least 2 valid codes.
    pub fn calculate_distance_matrix(&self, codes: &[&str]) -> Result<DistanceMatrix> {
        if codes.len() < 2 {
            return Err(AirportError::InvalidInput(
                "At least 2 airport codes are required for a distance matrix".to_string(),
            ));
        }

        let mut resolved: Vec<(&Airport, String)> = Vec::new();
        for code in codes {
            let airport = self.resolve_airport(code).map_err(|_| {
                AirportError::NotFound(format!("Airport not found for code: {}", code))
            })?;
            resolved.push((airport, code.to_string()));
        }

        let airport_infos: Vec<AirportInfo> = resolved
            .iter()
            .map(|(a, code)| AirportInfo {
                code: code.to_string(),
                name: a.airport.clone(),
                iata: a.iata.clone(),
                icao: a.icao.clone(),
            })
            .collect();

        let mut distances: HashMap<String, HashMap<String, f64>> = HashMap::new();
        for (a1, code1) in &resolved {
            let mut inner: HashMap<String, f64> = HashMap::new();
            for (a2, code2) in &resolved {
                if code1 == code2 {
                    inner.insert(code2.clone(), 0.0);
                } else {
                    let dist = haversine_distance(
                        a1.latitude, a1.longitude, a2.latitude, a2.longitude,
                    );
                    inner.insert(code2.clone(), dist.round());
                }
            }
            distances.insert(code1.clone(), inner);
        }

        Ok(DistanceMatrix {
            airports: airport_infos,
            distances,
        })
    }

    // =======================================================================
    // Validation & Utilities
    // =======================================================================

    /// Validates if an IATA code exists in the database.
    ///
    /// Code must be exactly 3 uppercase letters.
    pub fn validate_iata_code(&self, code: &str) -> bool {
        let trimmed = code.trim();
        if trimmed.len() != 3 || !trimmed.chars().all(|c| c.is_ascii_uppercase()) {
            return false;
        }
        AIRPORTS
            .iter()
            .any(|a| a.iata.eq_ignore_ascii_case(trimmed))
    }

    /// Validates if an ICAO code exists in the database.
    ///
    /// Code must be exactly 4 uppercase alphanumeric characters.
    pub fn validate_icao_code(&self, code: &str) -> bool {
        let trimmed = code.trim();
        if trimmed.len() != 4
            || !trimmed
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return false;
        }
        AIRPORTS
            .iter()
            .any(|a| a.icao.eq_ignore_ascii_case(trimmed))
    }

    /// Gets the count of airports matching the given filters.
    ///
    /// Pass `None` for total count of all airports.
    pub fn get_airport_count(&self, filter: Option<&AirportFilter>) -> usize {
        match filter {
            Some(f) => AIRPORTS.iter().filter(|a| matches_filter(a, f)).count(),
            None => AIRPORTS.len(),
        }
    }

    /// Checks if an airport has scheduled commercial service.
    ///
    /// Accepts either IATA or ICAO code.
    pub fn is_airport_operational(&self, code: &str) -> Result<bool> {
        let airport = self.resolve_airport(code)?;
        Ok(airport.scheduled_service)
    }

    // =======================================================================
    // Internal helpers
    // =======================================================================

    /// Resolves an airport code (tries IATA first, then ICAO).
    fn resolve_airport(&self, code: &str) -> Result<&Airport> {
        let trimmed = code.trim();
        // Try IATA first (3 chars)
        if let Some(a) = AIRPORTS
            .iter()
            .find(|a| !a.iata.is_empty() && a.iata.eq_ignore_ascii_case(trimmed))
        {
            return Ok(a);
        }
        // Try ICAO
        if let Some(a) = AIRPORTS
            .iter()
            .find(|a| !a.icao.is_empty() && a.icao.eq_ignore_ascii_case(trimmed))
        {
            return Ok(a);
        }
        Err(AirportError::NotFound(format!(
            "No airport found for code: {}",
            code
        )))
    }
}

impl Default for AirportData {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> AirportData {
        AirportData::new()
    }

    // ===================================================================
    // getAirportByIata
    // ===================================================================
    #[test]
    fn test_get_airport_by_iata_lhr() {
        let d = db();
        let airport = d.get_airport_by_iata("LHR").unwrap();
        assert_eq!(airport.iata, "LHR");
        assert!(airport.airport.contains("Heathrow"));
    }

    // ===================================================================
    // getAirportByIcao
    // ===================================================================
    #[test]
    fn test_get_airport_by_icao_egll() {
        let d = db();
        let airport = d.get_airport_by_icao("EGLL").unwrap();
        assert_eq!(airport.icao, "EGLL");
        assert!(airport.airport.contains("Heathrow"));
    }

    // ===================================================================
    // getAirportByCountryCode
    // ===================================================================
    #[test]
    fn test_get_airports_by_country_code_us() {
        let d = db();
        let airports = d.get_airports_by_country_code("US");
        assert!(airports.len() > 100);
        assert_eq!(airports[0].country_code, "US");
    }

    // ===================================================================
    // getAirportByContinent
    // ===================================================================
    #[test]
    fn test_get_airports_by_continent_eu() {
        let d = db();
        let airports = d.get_airports_by_continent("EU");
        assert!(airports.len() > 100);
        assert!(airports.iter().all(|a| a.continent == "EU"));
    }

    // ===================================================================
    // findNearbyAirports
    // ===================================================================
    #[test]
    fn test_find_nearby_airports_london() {
        let d = db();
        let nearby = d.find_nearby_airports(51.5074, -0.1278, 50.0);
        assert!(!nearby.is_empty());
        assert!(nearby.iter().any(|n| n.airport.iata == "LHR"));
    }

    // ===================================================================
    // getAirportsByType
    // ===================================================================
    #[test]
    fn test_get_airports_by_type_large() {
        let d = db();
        let airports = d.get_airports_by_type("large_airport");
        assert!(airports.len() > 10);
        assert!(airports.iter().all(|a| a.airport_type == "large_airport"));
    }

    #[test]
    fn test_get_airports_by_type_medium() {
        let d = db();
        let airports = d.get_airports_by_type("medium_airport");
        assert!(airports.len() > 10);
        assert!(airports.iter().all(|a| a.airport_type == "medium_airport"));
    }

    #[test]
    fn test_get_airports_by_type_airport_generic() {
        let d = db();
        let airports = d.get_airports_by_type("airport");
        assert!(airports.len() > 50);
        assert!(airports
            .iter()
            .all(|a| a.airport_type.contains("airport")));
    }

    #[test]
    fn test_get_airports_by_type_heliport() {
        let d = db();
        let heliports = d.get_airports_by_type("heliport");
        for h in &heliports {
            assert_eq!(h.airport_type, "heliport");
        }
    }

    #[test]
    fn test_get_airports_by_type_seaplane_base() {
        let d = db();
        let bases = d.get_airports_by_type("seaplane_base");
        for b in &bases {
            assert_eq!(b.airport_type, "seaplane_base");
        }
    }

    #[test]
    fn test_get_airports_by_type_case_insensitive() {
        let d = db();
        let upper = d.get_airports_by_type("LARGE_AIRPORT");
        let lower = d.get_airports_by_type("large_airport");
        assert_eq!(upper.len(), lower.len());
        assert!(!upper.is_empty());
    }

    #[test]
    fn test_get_airports_by_type_nonexistent() {
        let d = db();
        let airports = d.get_airports_by_type("nonexistent_type");
        assert!(airports.is_empty());
    }

    // ===================================================================
    // getAutocompleteSuggestions
    // ===================================================================
    #[test]
    fn test_autocomplete_london() {
        let d = db();
        let suggestions = d.get_autocomplete_suggestions("London");
        assert!(!suggestions.is_empty());
        assert!(suggestions.len() <= 10);
        assert!(suggestions.iter().any(|a| a.iata == "LHR"));
    }

    // ===================================================================
    // calculateDistance
    // ===================================================================
    #[test]
    fn test_calculate_distance_lhr_jfk() {
        let d = db();
        let dist = d.calculate_distance("LHR", "JFK").unwrap();
        // Approx 5541 km
        assert!((dist - 5541.0).abs() < 50.0);
    }

    // ===================================================================
    // findAirports (Advanced Filtering)
    // ===================================================================
    #[test]
    fn test_find_airports_gb_airport() {
        let d = db();
        let filter = AirportFilter {
            country_code: Some("GB".to_string()),
            airport_type: Some("airport".to_string()),
            ..Default::default()
        };
        let airports = d.find_airports(&filter);
        // The "airport" type matches large_airport, medium_airport, small_airport
        // so the airport_type field will contain "airport" as a substring
        assert!(airports
            .iter()
            .all(|a| a.country_code == "GB" && a.airport_type.contains("airport")));
    }

    #[test]
    fn test_find_airports_scheduled_service() {
        let d = db();
        let with_service = d.find_airports(&AirportFilter {
            has_scheduled_service: Some(true),
            ..Default::default()
        });
        let without_service = d.find_airports(&AirportFilter {
            has_scheduled_service: Some(false),
            ..Default::default()
        });
        assert!(with_service.len() + without_service.len() > 0);

        if !with_service.is_empty() {
            assert!(with_service.iter().all(|a| a.scheduled_service));
        }
        if !without_service.is_empty() {
            assert!(without_service.iter().all(|a| !a.scheduled_service));
        }
    }

    // ===================================================================
    // getAirportsByTimezone
    // ===================================================================
    #[test]
    fn test_get_airports_by_timezone_london() {
        let d = db();
        let airports = d.get_airports_by_timezone("Europe/London");
        assert!(airports.len() > 10);
        assert!(airports.iter().all(|a| a.timezone == "Europe/London"));
    }

    // ===================================================================
    // getAirportLinks
    // ===================================================================
    #[test]
    fn test_get_airport_links_lhr() {
        let d = db();
        let links = d.get_airport_links("LHR").unwrap();
        assert!(links
            .wikipedia
            .as_deref()
            .unwrap_or("")
            .contains("Heathrow_Airport"));
        assert!(links.website.is_some());
    }

    #[test]
    fn test_get_airport_links_hnd() {
        let d = db();
        let links = d.get_airport_links("HND").unwrap();
        assert!(links
            .wikipedia
            .as_deref()
            .unwrap_or("")
            .contains("Tokyo_International_Airport"));
        assert!(links.website.is_some());
    }

    // ===================================================================
    // getAirportStatsByCountry
    // ===================================================================
    #[test]
    fn test_stats_by_country_sg() {
        let d = db();
        let stats = d.get_airport_stats_by_country("SG").unwrap();
        assert!(stats.total > 0);
        assert!(!stats.timezones.is_empty());
    }

    #[test]
    fn test_stats_by_country_us() {
        let d = db();
        let stats = d.get_airport_stats_by_country("US").unwrap();
        assert!(stats.total > 1000);
        assert!(stats.by_type.contains_key("large_airport"));
        assert!(*stats.by_type.get("large_airport").unwrap() > 0);
    }

    #[test]
    fn test_stats_by_country_invalid() {
        let d = db();
        let result = d.get_airport_stats_by_country("XYZ");
        assert!(result.is_err());
    }

    // ===================================================================
    // getAirportStatsByContinent
    // ===================================================================
    #[test]
    fn test_stats_by_continent_as() {
        let d = db();
        let stats = d.get_airport_stats_by_continent("AS").unwrap();
        assert!(stats.total > 100);
        assert!(stats.by_country.len() > 10);
    }

    #[test]
    fn test_stats_by_continent_eu() {
        let d = db();
        let stats = d.get_airport_stats_by_continent("EU").unwrap();
        assert!(stats.by_country.contains_key("GB"));
        assert!(stats.by_country.contains_key("FR"));
        assert!(stats.by_country.contains_key("DE"));
    }

    // ===================================================================
    // getLargestAirportsByContinent
    // ===================================================================
    #[test]
    fn test_largest_by_continent_runway() {
        let d = db();
        let airports = d.get_largest_airports_by_continent("AS", 5, "runway");
        assert!(airports.len() <= 5);
        assert!(!airports.is_empty());
        for i in 0..airports.len() - 1 {
            let r1 = airports[i].runway_length.unwrap_or(0);
            let r2 = airports[i + 1].runway_length.unwrap_or(0);
            assert!(r1 >= r2);
        }
    }

    #[test]
    fn test_largest_by_continent_elevation() {
        let d = db();
        let airports = d.get_largest_airports_by_continent("SA", 5, "elevation");
        assert!(airports.len() <= 5);
        for i in 0..airports.len() - 1 {
            let e1 = airports[i].elevation_ft.unwrap_or(0);
            let e2 = airports[i + 1].elevation_ft.unwrap_or(0);
            assert!(e1 >= e2);
        }
    }

    #[test]
    fn test_largest_by_continent_respects_limit() {
        let d = db();
        let airports = d.get_largest_airports_by_continent("EU", 3, "runway");
        assert!(airports.len() <= 3);
    }

    // ===================================================================
    // getMultipleAirports
    // ===================================================================
    #[test]
    fn test_get_multiple_airports_iata() {
        let d = db();
        let airports = d.get_multiple_airports(&["SIN", "LHR", "JFK"]);
        assert_eq!(airports.len(), 3);
        assert_eq!(airports[0].as_ref().unwrap().iata, "SIN");
        assert_eq!(airports[1].as_ref().unwrap().iata, "LHR");
        assert_eq!(airports[2].as_ref().unwrap().iata, "JFK");
    }

    #[test]
    fn test_get_multiple_airports_mixed_codes() {
        let d = db();
        let airports = d.get_multiple_airports(&["SIN", "EGLL", "JFK"]);
        assert_eq!(airports.len(), 3);
        assert!(airports.iter().all(|a| a.is_some()));
    }

    #[test]
    fn test_get_multiple_airports_with_invalid() {
        let d = db();
        let airports = d.get_multiple_airports(&["SIN", "INVALID", "LHR"]);
        assert_eq!(airports.len(), 3);
        assert!(airports[0].is_some());
        assert!(airports[1].is_none());
        assert!(airports[2].is_some());
    }

    #[test]
    fn test_get_multiple_airports_empty() {
        let d = db();
        let airports = d.get_multiple_airports(&[]);
        assert!(airports.is_empty());
    }

    // ===================================================================
    // calculateDistanceMatrix
    // ===================================================================
    #[test]
    fn test_distance_matrix() {
        let d = db();
        let matrix = d
            .calculate_distance_matrix(&["SIN", "LHR", "JFK"])
            .unwrap();
        assert_eq!(matrix.airports.len(), 3);

        // Diagonal is zero
        assert_eq!(matrix.distances["SIN"]["SIN"], 0.0);
        assert_eq!(matrix.distances["LHR"]["LHR"], 0.0);
        assert_eq!(matrix.distances["JFK"]["JFK"], 0.0);

        // Symmetry
        assert_eq!(
            matrix.distances["SIN"]["LHR"],
            matrix.distances["LHR"]["SIN"]
        );
        assert_eq!(
            matrix.distances["SIN"]["JFK"],
            matrix.distances["JFK"]["SIN"]
        );

        // Reasonable distances
        assert!(matrix.distances["SIN"]["LHR"] > 5000.0);
        assert!(matrix.distances["LHR"]["JFK"] > 3000.0);
    }

    #[test]
    fn test_distance_matrix_too_few() {
        let d = db();
        let result = d.calculate_distance_matrix(&["SIN"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_distance_matrix_invalid_code() {
        let d = db();
        let result = d.calculate_distance_matrix(&["SIN", "INVALID"]);
        assert!(result.is_err());
    }

    // ===================================================================
    // findNearestAirport
    // ===================================================================
    #[test]
    fn test_find_nearest_airport_sin() {
        let d = db();
        let nearest = d
            .find_nearest_airport(1.35019, 103.994003, None)
            .unwrap();
        assert_eq!(nearest.airport.iata, "SIN");
        assert!(nearest.distance < 2.0);
    }

    #[test]
    fn test_find_nearest_airport_with_type_filter() {
        let d = db();
        let filter = AirportFilter {
            airport_type: Some("large_airport".to_string()),
            ..Default::default()
        };
        let nearest = d
            .find_nearest_airport(51.5074, -0.1278, Some(&filter))
            .unwrap();
        assert_eq!(nearest.airport.airport_type, "large_airport");
        assert!(nearest.distance > 0.0);
    }

    #[test]
    fn test_find_nearest_airport_with_type_and_country() {
        let d = db();
        let filter = AirportFilter {
            airport_type: Some("large_airport".to_string()),
            country_code: Some("US".to_string()),
            ..Default::default()
        };
        let nearest = d
            .find_nearest_airport(40.7128, -74.0060, Some(&filter))
            .unwrap();
        assert_eq!(nearest.airport.airport_type, "large_airport");
        assert_eq!(nearest.airport.country_code, "US");
    }

    // ===================================================================
    // validateIataCode
    // ===================================================================
    #[test]
    fn test_validate_iata_valid() {
        let d = db();
        assert!(d.validate_iata_code("SIN"));
        assert!(d.validate_iata_code("LHR"));
        assert!(d.validate_iata_code("JFK"));
    }

    #[test]
    fn test_validate_iata_invalid() {
        let d = db();
        assert!(!d.validate_iata_code("XYZ"));
        assert!(!d.validate_iata_code("ZZZ"));
    }

    #[test]
    fn test_validate_iata_bad_format() {
        let d = db();
        assert!(!d.validate_iata_code("ABCD"));
        assert!(!d.validate_iata_code("AB"));
        assert!(!d.validate_iata_code("abc"));
        assert!(!d.validate_iata_code(""));
    }

    // ===================================================================
    // validateIcaoCode
    // ===================================================================
    #[test]
    fn test_validate_icao_valid() {
        let d = db();
        assert!(d.validate_icao_code("WSSS"));
        assert!(d.validate_icao_code("EGLL"));
        assert!(d.validate_icao_code("KJFK"));
    }

    #[test]
    fn test_validate_icao_invalid() {
        let d = db();
        assert!(!d.validate_icao_code("XXXX"));
        assert!(!d.validate_icao_code("ZZZ0"));
    }

    #[test]
    fn test_validate_icao_bad_format() {
        let d = db();
        assert!(!d.validate_icao_code("ABC"));
        assert!(!d.validate_icao_code("ABCDE"));
        assert!(!d.validate_icao_code("abcd"));
        assert!(!d.validate_icao_code(""));
    }

    // ===================================================================
    // getAirportCount
    // ===================================================================
    #[test]
    fn test_get_airport_count_total() {
        let d = db();
        assert!(d.get_airport_count(None) > 5000);
    }

    #[test]
    fn test_get_airport_count_by_type() {
        let d = db();
        let large_count = d.get_airport_count(Some(&AirportFilter {
            airport_type: Some("large_airport".to_string()),
            ..Default::default()
        }));
        let total = d.get_airport_count(None);
        assert!(large_count > 0);
        assert!(large_count < total);
    }

    #[test]
    fn test_get_airport_count_by_country() {
        let d = db();
        let count = d.get_airport_count(Some(&AirportFilter {
            country_code: Some("US".to_string()),
            ..Default::default()
        }));
        assert!(count > 1000);
    }

    #[test]
    fn test_get_airport_count_multiple_filters() {
        let d = db();
        let count = d.get_airport_count(Some(&AirportFilter {
            country_code: Some("US".to_string()),
            airport_type: Some("large_airport".to_string()),
            ..Default::default()
        }));
        assert!(count > 0);
        assert!(count < 200);
    }

    // ===================================================================
    // isAirportOperational
    // ===================================================================
    #[test]
    fn test_is_airport_operational_true() {
        let d = db();
        assert!(d.is_airport_operational("SIN").unwrap());
        assert!(d.is_airport_operational("LHR").unwrap());
        assert!(d.is_airport_operational("JFK").unwrap());
    }

    #[test]
    fn test_is_airport_operational_both_codes() {
        let d = db();
        assert!(d.is_airport_operational("SIN").unwrap());
        assert!(d.is_airport_operational("WSSS").unwrap());
    }

    #[test]
    fn test_is_airport_operational_invalid() {
        let d = db();
        let result = d.is_airport_operational("INVALID");
        assert!(result.is_err());
    }

    // ===================================================================
    // searchByName
    // ===================================================================
    #[test]
    fn test_search_by_name() {
        let d = db();
        let results = d.search_by_name("Singapore").unwrap();
        assert!(!results.is_empty());
    }
}
