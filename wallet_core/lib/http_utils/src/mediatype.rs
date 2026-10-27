use std::fmt::Display;

use derive_more::Constructor;
use http::HeaderValue;
use http::header::ToStrError;
use mediatype::MediaTypeError;
use mediatype::MediaTypeList;
use mediatype::Name;
use mediatype::names::_STAR;

#[derive(thiserror::Error, Debug)]
pub enum AcceptError {
    #[error("non-ascii accept header: {0}")]
    NonAsciiHeader(#[source] ToStrError),

    #[error("invalid media type: {0}")]
    MediaType(#[source] MediaTypeError),
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct MediaType<'a> {
    /// Top-level type.
    ty: Name<'a>,

    /// Subtype.
    subty: Name<'a>,

    /// Optional suffix.
    suffix: Option<Name<'a>>,
}

impl<'a> From<mediatype::MediaType<'a>> for MediaType<'a> {
    fn from(value: mediatype::MediaType<'a>) -> Self {
        Self {
            ty: value.ty,
            subty: value.subty,
            suffix: value.suffix,
        }
    }
}

impl Display for MediaType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.ty, self.subty)?;
        if let Some(suffix) = self.suffix {
            write!(f, "+{}", suffix)?;
        }
        Ok(())
    }
}

const ALL_MEDIA_TYPE: MediaType = MediaType::new(_STAR, _STAR, None);

/// Parse accept header and find matching content type
///
/// When the accept header is missing or */* is specified it will return the default parameter.
/// Libraries like curl and reqwest will always specify an Accept header.
///
/// This method matches on order, ignoring quality values (q=). Reason is that for the current uses
/// the quality value probably won't be used at all and most clients sort it on decreasing value.
/// TODO: PVW-6102
pub fn find_content_type_from_accept<'a, T>(
    header: Option<&'a HeaderValue>,
    accept: impl Fn(MediaType<'a>) -> Option<T>,
    default: T,
) -> Result<Option<T>, AcceptError> {
    let Some(header) = header else { return Ok(Some(default)) };
    let header = header.to_str().map_err(AcceptError::NonAsciiHeader)?;

    let media_type_list = MediaTypeList::new(header);
    let mut all = false;

    for result in media_type_list {
        let media_type = MediaType::from(result.map_err(AcceptError::MediaType)?);
        if media_type == ALL_MEDIA_TYPE {
            all = true
        } else if let Some(accept) = accept(media_type) {
            return Ok(Some(accept));
        }
    }

    Ok(all.then_some(default))
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use mediatype::names::IMAGE;
    use mediatype::names::PLAIN;
    use mediatype::names::SVG;
    use mediatype::names::TEXT;
    use mediatype::names::XML;

    use super::*;

    #[test]
    fn empty_header() {
        let default = find_content_type_from_accept(None, |_| Some(true), false).unwrap();
        assert!(default.is_some());
        assert!(!default.unwrap());
    }

    fn match_text_plain(media_type: MediaType<'_>) -> Option<MediaType<'_>> {
        (media_type.ty == TEXT && media_type.subty == PLAIN).then_some(media_type)
    }

    #[test]
    fn matching_content_type() {
        let header = HeaderValue::from_str("text/html, text/plain").unwrap();
        let media_type =
            find_content_type_from_accept(Some(&header), match_text_plain, MediaType::new(TEXT, XML, None)).unwrap();
        assert!(media_type.is_some());
        assert_eq!(media_type.unwrap(), MediaType::new(TEXT, PLAIN, None));
    }

    #[test]
    fn non_matching_content_type() {
        let header = HeaderValue::from_str("text/html").unwrap();
        let media_type =
            find_content_type_from_accept(Some(&header), match_text_plain, MediaType::new(TEXT, XML, None)).unwrap();
        assert!(media_type.is_none());
    }

    #[test]
    fn non_matching_content_type_with_all() {
        let header = HeaderValue::from_str("text/html, */*;q=0.8").unwrap();
        let media_type =
            find_content_type_from_accept(Some(&header), match_text_plain, MediaType::new(TEXT, XML, None)).unwrap();
        assert!(media_type.is_some());
        assert_eq!(media_type.unwrap(), MediaType::new(TEXT, XML, None));
    }

    #[test]
    fn non_ascii_header() {
        let header = HeaderValue::from_str("text/ß").unwrap();
        let err = find_content_type_from_accept(Some(&header), |_| Some(()), ()).unwrap_err();
        assert_matches!(err, AcceptError::NonAsciiHeader(_));
    }

    #[test]
    fn invalid_accept_header() {
        let header = HeaderValue::from_str("hello world").unwrap();
        let err = find_content_type_from_accept(Some(&header), |_| Some(()), ()).unwrap_err();
        assert_matches!(err, AcceptError::MediaType(_));
    }

    #[test]
    fn display_media_type() {
        let media_type = MediaType::new(TEXT, XML, None);
        assert_eq!(media_type.to_string(), "text/xml");

        let media_type = MediaType::new(IMAGE, SVG, Some(XML));
        assert_eq!(media_type.to_string(), "image/svg+xml");
    }
}
