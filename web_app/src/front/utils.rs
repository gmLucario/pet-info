use chrono::{Datelike, NaiveDate};
use fast_qr::{
    convert::{image::ImageBuilder, Builder, Shape},
    qr::QRBuilder,
    ECL,
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
        .filter_map(|x| async move {
            if let Ok(b) = x {
                Some(b)
            } else {
                None
            }
        })
        .collect::<Vec<ntex::util::Bytes>>()
        .await
        .concat()
}

async fn get_bytes_as_str(
    x: Result<ntex::util::Bytes, ntex_multipart::MultipartError>,
) -> Option<String> {
    if let Ok(b) = x {
        if let Ok(value) = std::str::from_utf8(&b) {
            return Some(value.to_string());
        }
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

/// Human readable pet age
pub fn get_fmt_pet_age(birthday: NaiveDate, now: NaiveDate) -> String {
    let years = (now.year() - 1) - birthday.year();
    let months = now.month0().abs_diff(birthday.month0());

    if now.month0() < birthday.month0() {
        return format!(
            "{years} años y {months} meses",
            years = years,
            months = 12 - months
        );
    }

    if now.month0() > birthday.month0() {
        return format!(
            "{years} años y {months} meses",
            years = years + 1,
            months = months
        );
    }

    if now.day0() < birthday.day0() {
        return format!(
            "{years} años, y {months} meses y {days} dias",
            years = years,
            months = 11,
            days = now.day0() + 1,
        );
    }

    if now.day0() > birthday.day0() {
        return format!(
            "{years} años, y {days} dias",
            years = years + 1,
            days = now.day0().abs_diff(birthday.day0()),
        );
    }

    format!("{years} años", years = years + 1)
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
