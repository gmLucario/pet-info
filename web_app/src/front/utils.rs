use anyhow::Context;
use chrono::NaiveDate;
use chrono_tz::Tz;
use futures::StreamExt;

use crate::front;

/// Create HTTP redirect response
pub fn redirect_to(url: &str) -> Result<ntex::web::HttpResponse, ntex::web::Error> {
    Ok(ntex::web::HttpResponse::Found()
        .header("location", url)
        .finish())
}

/// Extract and concatenate bytes from multipart field
pub async fn get_bytes_value(field: ntex_multipart::Field) -> Vec<u8> {
    field
        .filter_map(|x| async move { x.ok() })
        .collect::<Vec<ntex::util::Bytes>>()
        .await
        .concat()
}

/// Convert bytes to UTF-8 string if valid
async fn get_bytes_as_str(
    x: Result<ntex::util::Bytes, ntex_multipart::MultipartError>,
) -> Option<String> {
    if let Ok(bytes) = x {
        return std::str::from_utf8(&bytes).ok().map(String::from);
    }

    None
}

/// Extract and concatenate string values from multipart field
pub async fn get_field_value(field: ntex_multipart::Field) -> String {
    field
        .filter_map(get_bytes_as_str)
        .collect::<Vec<String>>()
        .await
        .join("")
}

/// Extract and parse timezone from request headers
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

/// Format date difference as human-readable Spanish text
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

/// Get current UTC date at 00:00:00
pub fn get_utc_now_with_default_time() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
        .with_time(chrono::NaiveTime::default())
        .single()
        .unwrap()
}

/// Filter string to alphanumeric characters only
pub fn filter_only_alphanumeric_chars(s: &str) -> String {
    // Direct collection is more efficient than intermediate Vec
    s.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// Crop image into circular shape
pub fn crop_circle(
    pic: &front::forms::pet::Pic,
    x: u32,
    y: u32,
    diameter: u32,
) -> anyhow::Result<Vec<u8>> {
    let img_extension =
        image::ImageFormat::from_extension(&pic.filename_extension).context("invalid extension")?;
    let original_img = image::load_from_memory_with_format(&pic.body, img_extension)?;

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
mod tests {
    use super::*;

    #[test]
    fn test_extract_valid_usertimezone() -> anyhow::Result<()> {
        let vec = vec![("timezone", "America/Mexico_City"), ("Accept", "text/html")];
        let map = ntex::http::HeaderMap::from_iter(vec);

        let timezone = extract_usertimezone(&map)?;
        assert_eq!(timezone, chrono_tz::America::Mexico_City);

        Ok(())
    }

    #[test]
    fn test_redirect_to() {
        let redirect = redirect_to("/login");
        assert!(redirect.is_ok());

        let response = redirect.unwrap();
        assert_eq!(response.status(), ntex::http::StatusCode::FOUND);
        assert_eq!(response.headers().get("location").unwrap(), "/login");
    }

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

    #[test]
    fn test_get_utc_now_with_default_time() {
        let dt = get_utc_now_with_default_time();
        assert_eq!(dt.time(), chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

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

    #[test]
    fn test_extract_usertimezone_missing_header() {
        let vec = vec![("Accept", "text/html"), ("User-Agent", "test")];
        let map = ntex::http::HeaderMap::from_iter(vec);

        let result = extract_usertimezone(&map);
        assert!(result.is_err());
    }

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

        let pic = crate::front::forms::pet::Pic {
            body: img_data,
            filename_extension: "png".to_string(),
        };

        // Test basic cropping
        let result = crop_circle(&pic, 5, 5, 6);
        assert!(result.is_ok());

        let cropped_data = result.unwrap();
        assert!(!cropped_data.is_empty());

        // Verify it's PNG format
        assert_eq!(&cropped_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_crop_circle_invalid_format() {
        let pic = crate::front::forms::pet::Pic {
            body: vec![1, 2, 3, 4],                // Invalid image data
            filename_extension: "xyz".to_string(), // Invalid extension
        };

        let result = crop_circle(&pic, 5, 5, 6);
        assert!(result.is_err());
    }

    #[test]
    fn test_crop_circle_corrupted_data() {
        let pic = crate::front::forms::pet::Pic {
            body: vec![1, 2, 3, 4], // Invalid PNG data
            filename_extension: "png".to_string(),
        };

        let result = crop_circle(&pic, 5, 5, 6);
        assert!(result.is_err());
    }

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

    #[test]
    fn test_get_utc_now_time_zone_is_utc() {
        let dt = get_utc_now_with_default_time();
        assert_eq!(dt.timezone(), chrono::offset::Utc)
    }
}
