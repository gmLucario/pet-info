//! # Front end Utils
//!
//! Here are functions needed in all the front end app

use anyhow::bail;
use chrono::NaiveDate;
use chrono_tz::Tz;
use fast_qr::{
    ECL,
    convert::{Builder, Shape, image::ImageBuilder},
    qr::QRBuilder,
};
use futures::StreamExt;

/// [ntext responder](ntex::web::HttpResponse) to redirect to `url`
pub fn redirect_to(url: &str) -> Result<ntex::web::HttpResponse, ntex::web::Error> {
    Ok(ntex::web::HttpResponse::Found()
        .header("location", url)
        .finish())
}

/// Concats all the [bytes](Bytes) extracted from [Field]
pub async fn get_bytes_value(field: ntex_multipart::Field) -> Vec<u8> {
    field
        .filter_map(|x| async move { if let Ok(b) = x { Some(b) } else { None } })
        .collect::<Vec<ntex::util::Bytes>>()
        .await
        .concat()
}

async fn get_bytes_as_str(
    x: Result<ntex::util::Bytes, ntex_multipart::MultipartError>,
) -> Option<String> {
    if let Ok(Ok(v)) = x.map(|b| std::str::from_utf8(&b).map(|value| value.to_string())) {
        return Some(v);
    }

    None
}

/// Concats all the utf8 string values extracted from [Field]
pub async fn get_field_value(field: ntex_multipart::Field) -> String {
    field
        .filter_map(get_bytes_as_str)
        .collect::<Vec<String>>()
        .await
        .join("")
}

/// Extracts the 'timezone' header value from a [ntex::http::HeaderMap]
/// to cast it into a [chrono_tz::Tz]
pub fn extract_usertimezone(request_headers: &ntex::http::HeaderMap) -> anyhow::Result<Tz> {
    let user_timezone = request_headers
        .get("timezone")
        .map(|v| v.to_str().map(|tz| tz.parse::<Tz>()));

    if let Some(Ok(Ok(tz))) = user_timezone {
        return Ok(tz);
    }

    bail!("cant perse user time zone")
}

/// Human-readable dates difference.
/// It does not consider leap years. Months are taken as 30 days average
/// The output format is: x years y month u days
/// if x,y or u are zero, will be ignored
pub fn fmt_dates_difference(start_date: NaiveDate, end_date: NaiveDate) -> String {
    let (start_date, end_date) = if start_date <= end_date {
        (start_date, end_date)
    } else {
        (end_date, start_date)
    };

    let num_days = end_date.signed_duration_since(start_date).num_days();
    if num_days.lt(&1) {
        return "0 dîas".into();
    }

    let remaining_days = num_days % 365;
    let mut msg = String::new();

    let years = num_days / 365;
    let months = remaining_days / 30;
    let days = remaining_days % 30;

    if years > 0 {
        msg.push_str(&format!("{years} años"));
    }

    if months > 0 {
        msg.push_str(&format!(" {months} meses"));
    }

    if days > 0 {
        msg.push_str(&format!(" {} dîas", days - 1));
    }

    msg
}

pub fn get_utc_now_with_default_time() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
        .with_time(chrono::NaiveTime::default())
        .single()
        .unwrap()
}

pub fn get_qr_code(info_url_pat: String) -> anyhow::Result<Vec<u8>> {
    let qr_code = QRBuilder::new(info_url_pat.into_bytes())
        .ecl(ECL::H)
        .build()?;

    Ok(ImageBuilder::default()
        .shape(Shape::Square)
        .background_color("#ffffff") //hex value
        .module_color("#000000")
        .fit_width(600)
        .to_bytes(&qr_code)?)
}

pub fn filter_only_alphanumeric_chars(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
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
    fn test_raise_error_due_invalid_usertimezone() {
        let vec = vec![
            ("timezone", "America/Mexico_Citie"),
            ("Accept", "text/html"),
        ];
        let map = ntex::http::HeaderMap::from_iter(vec);

        let timezone = extract_usertimezone(&map);

        assert_eq!(timezone.is_err(), true);
    }
}
