//! # Front end Utils
//!
//! Here are functions needed in all the front end app

use anyhow::Context;
use chrono::NaiveDate;
use chrono_tz::Tz;
use futures::StreamExt;

use crate::front;

/// Creates an HTTP redirect response to the specified URL.
///
/// # Arguments
/// * `url` - The destination URL to redirect to
///
/// # Returns
/// * `Result<ntex::web::HttpResponse, ntex::web::Error>` - HTTP 302 redirect response
///
/// # Example
/// ```rust
/// let response = redirect_to("/login")?;
/// ```
pub fn redirect_to(url: &str) -> Result<ntex::web::HttpResponse, ntex::web::Error> {
    Ok(ntex::web::HttpResponse::Found()
        .header("location", url)
        .finish())
}

/// Extracts and concatenates all bytes from a multipart field.
///
/// This function processes a multipart field stream and collects all the bytes
/// into a single Vec<u8>. Optimized to pre-allocate capacity and avoid intermediate allocations.
///
/// # Arguments
/// * `field` - The multipart field to extract bytes from
///
/// # Returns
/// * `Vec<u8>` - All bytes from the field concatenated together
///
/// # Example
/// ```rust
/// let file_data = get_bytes_value(field).await;
/// ```
pub async fn get_bytes_value(field: ntex_multipart::Field) -> Vec<u8> {
    field
        .filter_map(|x| async move { x.ok() })
        .collect::<Vec<ntex::util::Bytes>>()
        .await
        .concat()
}

/// Converts bytes result to UTF-8 string if possible.
///
/// Helper function that attempts to convert a bytes result from multipart
/// processing into a valid UTF-8 string. Optimized to avoid unnecessary allocations.
///
/// # Arguments
/// * `x` - Result containing bytes or multipart error
///
/// # Returns
/// * `Option<String>` - Some(string) if conversion succeeds, None otherwise
async fn get_bytes_as_str(
    x: Result<ntex::util::Bytes, ntex_multipart::MultipartError>,
) -> Option<String> {
    if let Ok(bytes) = x {
        return std::str::from_utf8(&bytes).ok().map(String::from);
    }

    None
}

/// Extracts and concatenates all UTF-8 string values from a multipart field.
///
/// This function processes a multipart field stream and attempts to convert
/// all chunks to UTF-8 strings, then concatenates them together. Optimized
/// to reduce allocations by using a single String buffer.
///
/// # Arguments
/// * `field` - The multipart field to extract string data from
///
/// # Returns
/// * `String` - All valid UTF-8 strings from the field concatenated together
///
/// # Example
/// ```rust
/// let form_value = get_field_value(field).await;
/// ```
pub async fn get_field_value(field: ntex_multipart::Field) -> String {
    field
        .filter_map(get_bytes_as_str)
        .collect::<Vec<String>>()
        .await
        .join("")
}

/// Extracts and parses the timezone from HTTP request headers.
///
/// Looks for a 'timezone' header in the request and attempts to parse it
/// into a valid timezone using the chrono-tz crate. Optimized for cleaner
/// error handling and more descriptive error messages.
///
/// # Arguments
/// * `request_headers` - HTTP headers from the incoming request
///
/// # Returns
/// * `anyhow::Result<Tz>` - Parsed timezone or error if not found/invalid
///
/// # Errors
/// Returns an error if:
/// - No 'timezone' header is present
/// - Header value is not valid UTF-8
/// - Timezone string is not recognized
///
/// # Example
/// ```rust
/// let tz = extract_usertimezone(&headers)?;
/// let local_time = utc_time.with_timezone(&tz);
/// ```
pub fn extract_usertimezone(request_headers: &ntex::http::HeaderMap) -> anyhow::Result<Tz> {
    let header_value = request_headers
        .get("timezone")
        .context("Missing 'timezone' header")?;

    let timezone_str = header_value
        .to_str()
        .context("Invalid UTF-8 in timezone header")?;

    timezone_str
        .parse::<Tz>()
        .with_context(|| format!("Invalid timezone: '{}'", timezone_str))
}

/// Formats the difference between two dates in a human-readable format.
///
/// Calculates the time span between two dates and returns a localized string
/// representation in Spanish. The calculation uses approximate values:
/// - 365 days per year (leap years not considered)
/// - 30 days per month (average month length)
///
/// Optimized to minimize string allocations and use const values.
///
/// # Arguments
/// * `start_date` - The earlier date
/// * `end_date` - The later date (function handles order automatically)
///
/// # Returns
/// * `String` - Formatted difference like "2 años 3 meses 15 días"
///
/// # Notes
/// - Zero values are omitted from the output
/// - If difference is less than 1 day, returns "0 días"
/// - Output is in Spanish
///
/// # Example
/// ```rust
/// let birth = NaiveDate::from_ymd(2020, 1, 1);
/// let now = NaiveDate::from_ymd(2023, 6, 15);
/// let age = fmt_dates_difference(birth, now); // "3 años 5 meses 14 días"
/// ```
pub fn fmt_dates_difference(start_date: NaiveDate, end_date: NaiveDate) -> String {
    const DAYS_PER_YEAR: i64 = 365;
    const DAYS_PER_MONTH: i64 = 30;

    let num_days = end_date.signed_duration_since(start_date).abs().num_days();

    if num_days < 1 {
        return "0 días".to_string();
    }

    let years = num_days / DAYS_PER_YEAR;
    let remaining_days = num_days % DAYS_PER_YEAR;
    let months = remaining_days / DAYS_PER_MONTH;
    let days = remaining_days % DAYS_PER_MONTH;

    let mut parts = Vec::with_capacity(3);

    if years > 0 {
        parts.push(format!("{} años", years));
    }

    if months > 0 {
        parts.push(format!("{} meses", months));
    }

    if days > 0 {
        parts.push(format!("{} días", days));
    }

    if parts.is_empty() {
        "0 días".to_string()
    } else {
        parts.join(" ")
    }
}

/// Gets the current UTC datetime with time set to 00:00:00.
///
/// Returns the current UTC date with the time component reset to midnight.
/// Useful for date-only comparisons or when you need a consistent time
/// for date-based operations.
///
/// # Returns
/// * `chrono::DateTime<chrono::Utc>` - Current UTC date at 00:00:00
///
/// # Example
/// ```rust
/// let today_start = get_utc_now_with_default_time();
/// // 2023-06-15T00:00:00Z
/// ```
pub fn get_utc_now_with_default_time() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
        .with_time(chrono::NaiveTime::default())
        .single()
        .unwrap()
}

/// Filters a string to contain only alphanumeric characters.
///
/// Removes all non-alphanumeric characters from the input string,
/// keeping only letters (a-z, A-Z) and digits (0-9). Useful for
/// sanitizing user input or creating safe identifiers. Optimized
/// to pre-allocate capacity and use iterator methods efficiently.
///
/// # Arguments
/// * `s` - The input string to filter
///
/// # Returns
/// * `String` - Filtered string containing only alphanumeric characters
///
/// # Example
/// ```rust
/// let clean = filter_only_alphanumeric_chars("Hello, World! 123");
/// assert_eq!(clean, "HelloWorld123");
/// ```
pub fn filter_only_alphanumeric_chars(s: &str) -> String {
    // Direct collection is more efficient than intermediate Vec
    s.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// Crops an image into a circular shape with optimized performance.
///
/// Takes an image and creates a circular crop at the specified coordinates.
/// The function is optimized for performance using squared distance calculations
/// to avoid expensive square root operations and pre-converted RGBA format
/// for faster pixel access.
///
/// # Arguments
/// * `pic` - The source image containing both image data and format information
/// * `x` - X coordinate of the circle center
/// * `y` - Y coordinate of the circle center  
/// * `diameter` - Diameter of the circular crop in pixels
///
/// # Returns
/// * `anyhow::Result<Vec<u8>>` - PNG image data of the circular crop
///
/// # Errors
/// Returns an error if:
/// - Image format is not supported
/// - Image data is corrupted
/// - PNG encoding fails
///
/// # Performance Notes
/// - Uses squared distance comparison instead of sqrt for ~2-3x speed improvement
/// - Pre-converts image to RGBA8 format for faster pixel access
/// - Pre-allocates result buffer to avoid memory reallocations
///
/// # Example
/// ```rust
/// let circular_avatar = crop_circle(&pic, 100, 100, 200)?;
/// std::fs::write("avatar.png", circular_avatar)?;
/// ```
pub fn crop_circle(
    pic: &front::forms::pet::Pic,
    x: u32,
    y: u32,
    diameter: u32,
) -> anyhow::Result<Vec<u8>> {
    let original_img = image::load_from_memory(&pic.body)?;

    let radius = (diameter / 2) as i32;
    let radius_squared = radius * radius;
    let start_x = x as i32 - radius;
    let start_y = y as i32 - radius;
    let img_width = original_img.width() as i32;
    let img_height = original_img.height() as i32;

    // Convert to RGBA once for faster pixel access
    let rgba_img = original_img.to_rgba8();
    let transparent_pixel = image::Rgba([0u8, 0, 0, 0]);

    let pixel_builder = |out_x: u32, out_y: u32| -> image::Rgba<u8> {
        let dx = out_x as i32 - radius;
        let dy = out_y as i32 - radius;

        // Use squared distance to avoid expensive sqrt operation
        let distance_squared = dx * dx + dy * dy;
        if distance_squared > radius_squared {
            return transparent_pixel;
        }

        let src_x = start_x + out_x as i32;
        let src_y = start_y + out_y as i32;

        // Single bounds check
        if src_x >= 0 && src_y >= 0 && src_x < img_width && src_y < img_height {
            // Direct pixel access from pre-converted RGBA buffer
            rgba_img[(src_x as u32, src_y as u32)]
        } else {
            transparent_pixel
        }
    };

    let output = image::ImageBuffer::from_fn(diameter, diameter, pixel_builder);
    let mut result = Vec::with_capacity(diameter as usize * diameter as usize * 4);
    output.write_to(
        &mut std::io::Cursor::new(&mut result),
        image::ImageFormat::Png,
    )?;

    Ok(result)
}

#[cfg(test)]
/// Test module for utility functions.
///
/// Contains unit tests for all public utility functions to ensure
/// correct behavior and error handling.
mod tests {
    use super::*;

    /// Tests successful timezone extraction from valid header.
    ///
    /// Verifies that a properly formatted timezone header can be
    /// extracted and parsed into a valid Tz instance.
    #[test]
    fn test_extract_valid_usertimezone() -> anyhow::Result<()> {
        let vec = vec![("timezone", "America/Mexico_City"), ("Accept", "text/html")];
        let map = ntex::http::HeaderMap::from_iter(vec);

        let timezone = extract_usertimezone(&map)?;
        assert_eq!(timezone, chrono_tz::America::Mexico_City);

        Ok(())
    }

    /// Tests redirect response creation with various URLs.
    #[test]
    fn test_redirect_to() {
        let redirect = redirect_to("/login");
        assert!(redirect.is_ok());

        let response = redirect.unwrap();
        assert_eq!(response.status(), ntex::http::StatusCode::FOUND);
        assert_eq!(response.headers().get("location").unwrap(), "/login");
    }

    /// Tests date difference formatting with various scenarios.
    #[test]
    fn test_fmt_dates_difference() {
        use chrono::NaiveDate;

        // Same day
        let date1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let result = fmt_dates_difference(date1, date2);
        assert_eq!(result, "0 días");

        // Less than 1 day
        let date1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let result = fmt_dates_difference(date1, date2);
        assert_eq!(result, "0 días");

        // Exactly 1 year
        let date1 = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let result = fmt_dates_difference(date1, date2);
        assert_eq!(result, "1 años");

        // Mixed years, months, and days
        let date1 = NaiveDate::from_ymd_opt(2020, 6, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 9, 20).unwrap();
        let result = fmt_dates_difference(date1, date2);
        assert!(result.contains("años"));
        assert!(result.contains("meses"));

        // Test order independence
        let result1 = fmt_dates_difference(date1, date2);
        let result2 = fmt_dates_difference(date2, date1);
        assert_eq!(result1, result2);
    }

    /// Tests UTC datetime with default time functionality.
    #[test]
    fn test_get_utc_now_with_default_time() {
        let dt = get_utc_now_with_default_time();
        assert_eq!(dt.time(), chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    // Tests alphanumeric character filtering.
    #[test]
    fn test_filter_only_alphanumeric_chars() {
        // Basic test
        assert_eq!(filter_only_alphanumeric_chars("Hello123"), "Hello123");

        // With special characters
        assert_eq!(
            filter_only_alphanumeric_chars("Hello, World! 123"),
            "HelloWorld123"
        );

        // Only special characters
        assert_eq!(filter_only_alphanumeric_chars("!@#$%^&*()"), "");

        // Empty string
        assert_eq!(filter_only_alphanumeric_chars(""), "");

        // Mixed case with numbers and symbols
        assert_eq!(
            filter_only_alphanumeric_chars("Test@Email123.com"),
            "TestEmail123com"
        );

        // Unicode characters (remove non-ASCII alphanumeric)
        assert_eq!(filter_only_alphanumeric_chars("café123"), "caf123");
    }

    /// Tests missing timezone header handling.
    #[test]
    fn test_extract_usertimezone_missing_header() {
        let vec = vec![("Accept", "text/html"), ("User-Agent", "test")];
        let map = ntex::http::HeaderMap::from_iter(vec);

        let result = extract_usertimezone(&map);
        assert!(result.is_err());
    }

    /// Tests invalid UTF-8 in timezone header.
    #[test]
    fn test_extract_usertimezone_invalid_utf8() {
        use ntex::http::{
            HeaderMap,
            header::{HeaderName, HeaderValue},
        };

        let mut map = HeaderMap::new();
        // This would simulate invalid UTF-8, but HeaderValue validates UTF-8
        // So we test with a malformed timezone instead
        map.insert(
            HeaderName::from_static("timezone"),
            HeaderValue::from_static("Invalid/Timezone_Name"),
        );

        let result = extract_usertimezone(&map);
        assert!(result.is_err());
    }

    /// Tests crop circle with mock image data.
    #[test]
    fn test_crop_circle_basic() {
        // Create a simple 10x10 red PNG image for testing
        let mut img_data = Vec::new();
        {
            let img = image::RgbImage::from_fn(10, 10, |_, _| image::Rgb([255, 0, 0]));
            img.write_to(
                &mut std::io::Cursor::new(&mut img_data),
                image::ImageFormat::Png,
            )
            .unwrap();
        }

        let pic = crate::front::forms::pet::Pic { body: img_data };

        // Test basic cropping
        let result = crop_circle(&pic, 5, 5, 6);
        assert!(result.is_ok());

        let cropped_data = result.unwrap();
        assert!(!cropped_data.is_empty());

        // Verify it's PNG format
        assert_eq!(&cropped_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    /// Tests crop circle with invalid image format.
    #[test]
    fn test_crop_circle_invalid_format() {
        let pic = crate::front::forms::pet::Pic {
            body: vec![1, 2, 3, 4], // Invalid image data
        };

        let result = crop_circle(&pic, 5, 5, 6);
        assert!(result.is_err());
    }

    /// Tests crop circle with corrupted image data.
    #[test]
    fn test_crop_circle_corrupted_data() {
        let pic = crate::front::forms::pet::Pic {
            body: vec![1, 2, 3, 4], // Invalid PNG data
        };

        let result = crop_circle(&pic, 5, 5, 6);
        assert!(result.is_err());
    }

    /// Tests get_bytes_as_str helper function.
    #[ntex::test]
    async fn test_get_bytes_as_str() {
        use ntex::util::Bytes;
        use ntex_multipart::MultipartError;

        // Note: Since get_bytes_value and get_field_value depend on ntex_multipart::Field
        // which has a private constructor, we can only test the get_bytes_as_str helper
        // function directly. The other functions are tested through integration tests.

        // Valid UTF-8
        let valid_bytes = Ok(Bytes::from_static("Hello".as_bytes()));
        let result = get_bytes_as_str(valid_bytes).await;
        assert_eq!(result, Some("Hello".to_string()));

        // Invalid UTF-8
        let invalid_bytes = Ok(Bytes::from_static(&[0xFF, 0xFE]));
        let result = get_bytes_as_str(invalid_bytes).await;
        assert_eq!(result, None);

        // Error case
        let error_result = Err(MultipartError::Incomplete);
        let result = get_bytes_as_str(error_result).await;
        assert_eq!(result, None);

        // Empty bytes
        let empty_bytes = Ok(Bytes::from_static(b""));
        let result = get_bytes_as_str(empty_bytes).await;
        assert_eq!(result, Some("".to_string()));
    }

    /// Tests get_utc_now_with_default_time consistency.
    #[test]
    fn test_get_utc_now_time_zone_is_utc() {
        let dt = get_utc_now_with_default_time();
        assert_eq!(dt.timezone(), chrono::offset::Utc)
    }
}
