use http::HeaderValue;
use http::header::ToStrError;
use itertools::Itertools;
use mediatype::MediaType;
use mediatype::MediaTypeError;
use mediatype::MediaTypeList;
use mediatype::Name;

pub const ALL_MEDIA_TYPE: MediaType = MediaType::new(Name::new_unchecked("*"), Name::new_unchecked("*"));

#[derive(thiserror::Error, Debug)]
pub enum AcceptError {
    #[error("non-ascii accept header: {0}")]
    NonAsciiHeader(#[source] ToStrError),

    #[error("invalid media type: {0}")]
    MediaType(#[source] MediaTypeError),
}

/// Parse accept header and find matching content type
pub fn find_content_type_from_accept<'a, T: 'a>(
    header: &'a HeaderValue,
    accept: impl Fn(MediaType<'a>) -> Option<T>,
) -> Result<Option<T>, AcceptError> {
    let header = header.to_str().map_err(AcceptError::NonAsciiHeader)?;
    MediaTypeList::new(header)
        .process_results(|iter| iter.into_iter().filter_map(accept).next())
        .map_err(AcceptError::MediaType)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn matching_content_type() {
        let header = HeaderValue::from_str("text/html, text/plain").unwrap();
        let content_type = find_content_type_from_accept(&header, |media_type| {
            (media_type.ty == "text" && media_type.subty == "plain").then_some(media_type)
        })
        .unwrap();
        assert!(content_type.is_some());
        assert_eq!(content_type.unwrap().subty, "plain");
    }

    #[test]
    fn non_matching_content_type() {
        let header = HeaderValue::from_str("text/html").unwrap();
        let content_type = find_content_type_from_accept(&header, |media_type| {
            (media_type.ty == "text" && media_type.subty == "plain").then_some(media_type)
        })
        .unwrap();
        assert!(content_type.is_none());
    }

    #[test]
    fn non_ascii_header() {
        let header = HeaderValue::from_str("text/ß").unwrap();
        let err = find_content_type_from_accept(&header, |_| Some(())).unwrap_err();
        assert_matches!(err, AcceptError::NonAsciiHeader(_));
    }

    #[test]
    fn invalid_accept_header() {
        let header = HeaderValue::from_str("hello world").unwrap();
        let err = find_content_type_from_accept(&header, |_| Some(())).unwrap_err();
        assert_matches!(err, AcceptError::MediaType(_));
    }
}
