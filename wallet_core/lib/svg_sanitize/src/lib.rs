mod allow;

use derive_more::AsRef;
use derive_more::Into;
use quick_xml::Reader;
use quick_xml::Writer;
use quick_xml::events::BytesStart;
use quick_xml::events::BytesText;
use quick_xml::events::Event;

use crate::allow::LowerCaseString;
use crate::allow::UrlUnescapedString;

/// Errors that can occur during sanitization.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("escape error: {0}")]
    Escape(#[from] quick_xml::escape::EscapeError),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

/// A sanitized SVG.
///
/// The sanitizer uses an allowlist approach:
/// - Only known-safe tags and attributes are passed through.
/// - `href`, `xlink:href`, and `src` values are validated against a strict URL allowlist;
///   only fragment refs (`#…`) and inert raster data URIs are permitted — no external URLs.
/// - `<!DOCTYPE>` declarations are dropped entirely, preventing entity expansion attacks.
/// - Comments and processing instructions are dropped.
/// - CDATA sections are converted to escaped text nodes.
/// - `<use>`, `<animate>`, `<set>`, `<script>`, `<style>`, and `<foreignObject>` are blocked.
///
/// # Known limitation
/// The `style` attribute is passed through without CSS sanitization. Avoid this crate
/// if you need to block CSS-based attacks (e.g. `expression()`, `url()`). CSS sanitization
/// may be added in a future version.
#[derive(Clone, Debug, AsRef, Into)]
pub struct SanitizedSvg(String);

impl SanitizedSvg {
    pub fn try_new(xml: &str) -> Result<Self, Error> {
        Ok(Self(sanitize(xml)?))
    }
}

fn sanitize(input: &str) -> Result<String, Error> {
    let mut reader = Reader::from_str(input);
    {
        let cfg = reader.config_mut();
        // Do not synthesize Start+End pairs for self-closing tags; we handle
        // Empty events explicitly so skip_depth stays correct.
        cfg.expand_empty_elements = false;
        // Tolerate mismatched closing tags common in real-world SVG.
        cfg.check_end_names = false;
        // Tolerate malformed comments (e.g. `-- ` inside) in adversarial input.
        cfg.check_comments = false;
    }

    let mut writer = Writer::new(Vec::new());

    // When skip_depth > 0 we are inside a blocked element; all events are discarded
    // until the matching close tag brings it back to 0.
    let mut skip_depth: usize = 0;

    loop {
        match reader.read_event()? {
            Event::Eof => break,

            // Drop DOCTYPE entirely. quick-xml is a pure-Rust non-expanding parser,
            // so user-defined entities would surface as errors rather than expansions,
            // but we drop DOCTYPE anyway to prevent any future confusion and to make
            // the policy explicit.
            Event::DocType(_) => {}

            // Drop comments — they can hide payloads in some parser combinations.
            Event::Comment(_) => {}

            // Drop processing instructions (e.g. <?xml-stylesheet href="evil"?>).
            Event::PI(_) => {}

            // Pass the XML declaration through; it is safe and often required.
            Event::Decl(d) => {
                if skip_depth == 0 {
                    writer.write_event(Event::Decl(d))?;
                }
            }

            Event::Text(t) => {
                if skip_depth == 0 {
                    writer.write_event(Event::Text(t))?;
                }
            }

            // Convert CDATA to escaped text so downstream parsers see a plain text
            // node rather than a raw CDATA section, which some parsers treat as a
            // script-injection opportunity.
            Event::CData(cd) => {
                if skip_depth == 0 {
                    let raw = str::from_utf8(cd.as_ref())?;
                    let text = BytesText::new(raw);
                    writer.write_event(Event::Text(text))?;
                }
            }

            Event::Start(e) => {
                if skip_depth > 0 {
                    skip_depth += 1;
                } else {
                    match filter_element(&e)? {
                        Some(clean) => writer.write_event(Event::Start(clean))?,
                        None => skip_depth += 1,
                    }
                }
            }

            // Self-closing tags: no matching End event, so skip_depth is unchanged
            // whether we allow or block the element.
            Event::Empty(e) => {
                if skip_depth == 0
                    && let Some(clean) = filter_element(&e)?
                {
                    writer.write_event(Event::Empty(clean))?;
                }
            }

            Event::End(e) => {
                if skip_depth > 0 {
                    skip_depth -= 1;
                } else {
                    writer.write_event(Event::End(e))?;
                }
            }

            // Drop entity references (&amp;, &lol;, &#65;, etc.). quick-xml
            // does not expand them — it just emits the name — so there is no
            // DoS risk, but passing them through would write bare &name; tokens
            // into the output, producing invalid XML for any entity that is not
            // predefined. Dropping is the conservative choice.
            Event::GeneralRef(_) => {}
        }
    }

    let bytes = writer.into_inner();
    let result = str::from_utf8(&bytes).map_err(Error::Utf8)?;
    Ok(result.to_string())
}

/// Returns a sanitized copy of the element's `BytesStart` if the tag is allowed,
/// or `None` if the entire element (and its children) should be skipped.
fn filter_element(e: &BytesStart<'_>) -> Result<Option<BytesStart<'static>>, Error> {
    let name_bytes = e.local_name();
    let name_str = str::from_utf8(name_bytes.as_ref())?;
    let tag_name = LowerCaseString::new(name_str);

    if !allow::is_allowed_tag(&tag_name) {
        return Ok(None);
    }

    // Preserve original tag-name casing (SVG/XML is case-sensitive).
    let mut out = BytesStart::new(name_str.to_owned());

    for attr_result in e.attributes().with_checks(false) {
        let attr = attr_result.map_err(|e| Error::Xml(quick_xml::Error::InvalidAttr(e)))?;
        let attr_name = LowerCaseString::new(str::from_utf8(attr.key.as_ref())?);

        if !(allow::is_allowed_attr(&attr_name) || allow::is_allowed_by_prefix(&attr_name)) {
            continue;
        }

        if allow::is_url_attr(&attr_name) {
            if !allow::is_safe_url(&UrlUnescapedString::new(&attr)?) {
                continue;
            }
        }

        out.push_attribute(attr);
    }

    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::sanitize;

    fn sanitize_panicking(input: &str) -> String {
        sanitize(input).expect("sanitize failed")
    }

    // ── Tag allowlist — including HTML elements and non-ASCII names ───────────

    #[test]
    fn script_tag_removed() {
        let out = sanitize_panicking(r#"<svg><script>alert(1)</script><circle r="5"/></svg>"#);
        assert!(!out.contains("script"), "got: {out}");
        assert!(!out.contains("alert"), "got: {out}");
        assert!(out.contains("<circle"), "got: {out}");
    }

    #[test]
    fn script_tag_uppercase_removed() {
        // Tag matching must be case-insensitive; <SCRIPT> is just as dangerous as <script>.
        let out = sanitize_panicking(r#"<svg><SCRIPT>alert(1)</SCRIPT><circle r="5"/></svg>"#);
        assert!(!out.contains("SCRIPT"), "got: {out}");
        assert!(!out.contains("alert"), "got: {out}");
        assert!(out.contains("<circle"), "got: {out}");
    }

    #[test]
    fn script_tag_children_also_removed() {
        // Children of a blocked element must be dropped even if they are themselves allowed tags.
        let out = sanitize_panicking(r#"<svg><script><circle r="5"/></script></svg>"#);
        assert!(!out.contains("script"), "got: {out}");
        // The circle is inside the script block — it must not leak into the output.
        assert!(!out.contains("<circle"), "got: {out}");
    }

    #[test]
    fn use_tag_blocked() {
        // Non-self-closing <use> (Start + End events).
        let out = sanitize_panicking(r##"<svg><defs><circle id="c" r="10"/></defs><use href="#c"></use></svg>"##);
        assert!(!out.contains("<use"), "got: {out}");
        assert!(out.contains("<circle"), "got: {out}");
    }

    #[test]
    fn use_tag_self_closing_blocked() {
        // Self-closing <use/> takes the Empty event path, not Start/End.
        let out = sanitize_panicking(r##"<svg><defs><circle id="c" r="10"/></defs><use href="#c"/></svg>"##);
        assert!(!out.contains("<use"), "got: {out}");
        assert!(out.contains("<circle"), "got: {out}");
    }

    #[test]
    fn set_tag_blocked() {
        let out = sanitize_panicking(r#"<svg><rect><set attributeName="href" to="javascript:alert(1)"/></rect></svg>"#);
        assert!(!out.contains("<set"), "got: {out}");
    }

    #[test]
    fn foreign_object_blocked() {
        let out = sanitize_panicking(r#"<svg><foreignObject><div>hi</div></foreignObject></svg>"#);
        assert!(!out.contains("foreignObject"), "got: {out}");
        assert!(!out.contains("<div"), "got: {out}");
    }

    #[test]
    fn animate_blocked() {
        let out =
            sanitize_panicking(r#"<svg><rect><animate attributeName="href" to="javascript:alert(1)"/></rect></svg>"#);
        assert!(!out.contains("animate"), "got: {out}");
    }

    #[test]
    fn style_element_blocked() {
        let out = sanitize_panicking(r#"<svg><style>* { fill: red }</style><rect/></svg>"#);
        assert!(!out.contains("<style"), "got: {out}");
    }

    #[test]
    fn html_form_blocked() {
        // <form> and <input> are HTML elements with no place in SVG.
        let out = sanitize_panicking(
            r#"<svg><form action="javascript:alert(1)"><input type="submit" onclick="alert(1)"/></form></svg>"#,
        );
        assert!(!out.contains("<form"), "got: {out}");
        assert!(!out.contains("<input"), "got: {out}");
    }

    #[test]
    fn non_ascii_prefixed_tag_blocked() {
        // Non-ASCII namespace prefixes should not bypass the tag allowlist.
        let out = sanitize_panicking(r#"<svg><ø:script src="//evil.example/">x</ø:script></svg>"#);
        assert!(!out.contains("script"), "got: {out}");
    }

    // ── Attribute allowlist ────────────────────────────────────────────────────

    /// Event handler attributes (`onclick`, `onload`, etc.) must be stripped
    /// regardless of case. The safe sibling attribute `r="5"` must survive.
    #[rstest]
    #[case("onclick")]
    #[case("onClick")]
    #[case("onload")]
    #[case("ONLOAD")]
    fn event_handler_stripped(#[case] attr: &str) {
        let svg = format!(r#"<svg><circle {attr}="alert(1)" r="5"/></svg>"#);
        let out = sanitize_panicking(&svg);
        assert!(!out.contains(attr), "got: {out}");
        assert!(out.contains(r#"r="5""#), "got: {out}");
    }

    // ── href / URL validation ──────────────────────────────────────────────────

    /// These href values must be stripped entirely because they don't match the
    /// positive URL allowlist (fragment-ref or an explicit raster data: MIME type).
    #[rstest]
    #[case("javascript:alert(1)")]
    #[case("vbscript:MsgBox(1)")]
    #[case("http://example.com")]
    #[case("https://example.com")] // external URLs blocked to prevent user tracking
    #[case("/images/logo.svg")]
    #[case("data:text/html,xss")]
    #[case("data:image/svg+xml;base64,PHN2Zy8+")] // SVG-in-SVG executes as same-origin
    fn href_blocked(#[case] href: &str) {
        let svg = format!(r#"<svg><a href="{href}">x</a></svg>"#);
        let out = sanitize_panicking(&svg);
        assert!(!out.contains(href), "got: {out}");
    }

    /// These href values must pass through unchanged.
    #[rstest]
    #[case("#target")]
    #[case("data:image/png;base64,iVBORw0KGgo=")]
    #[case("data:image/jpeg;base64,/9j/4AAQ=")]
    #[case("data:image/gif;base64,R0lGODlh")]
    #[case("data:image/webp;base64,UklGRg==")]
    fn href_allowed(#[case] href: &str) {
        let svg = format!(r#"<svg><a href="{href}">x</a></svg>"#);
        let out = sanitize_panicking(&svg);
        assert!(out.contains(href), "got: {out}");
    }

    // ── href bypass vectors (tested individually because each targets a specific
    //    obfuscation technique and deserves a named, documented test) ──────────

    #[test]
    fn javascript_xlink_href_blocked() {
        let out = sanitize_panicking(
            r#"<svg xmlns:xlink="http://www.w3.org/1999/xlink"><a xlink:href="javascript:alert(1)"><text>x</text></a></svg>"#,
        );
        assert!(!out.contains("javascript"), "got: {out}");
    }

    #[test]
    fn javascript_src_blocked() {
        let out = sanitize_panicking(r#"<svg><image src="javascript:alert(1)"/></svg>"#);
        assert!(!out.contains("javascript"), "got: {out}");
    }

    #[test]
    fn encoded_javascript_href_blocked() {
        // `java&#115;cript:` decodes to `javascript:` — must be caught after unescaping.
        let out = sanitize_panicking(r#"<svg><a href="java&#115;cript:alert(1)">x</a></svg>"#);
        assert!(!out.contains("javascript"), "got: {out}");
        assert!(!out.contains("&#115;"), "got: {out}");
    }

    #[test]
    fn tab_encoded_javascript_href_blocked() {
        // `javascript&#9;:` inserts a tab between "javascript" and ":".
        // Some browsers strip whitespace in href values before interpreting the
        // protocol, making this a valid XSS vector. Our positive allowlist blocks
        // it because the decoded value doesn't start with "#" or "https://".
        let out = sanitize_panicking(r#"<svg><a href="javascript&#9;:alert(1)">x</a></svg>"#);
        assert!(!out.contains("javascript"), "got: {out}");
    }

    #[test]
    fn xml_base_blocked() {
        // xml:base rebases relative URI resolution: a browser resolves href="#x"
        // against the base, yielding `javascript:alert(1)//#x`. Strip xml:base
        // entirely rather than trying to validate it as a URL.
        let out = sanitize_panicking(r##"<svg><a xml:base="javascript:alert(1)//" href="#x">click</a></svg>"##);
        assert!(!out.contains("xml:base"), "got: {out}");
    }

    #[test]
    fn double_data_prefix_href_blocked() {
        // `data:data:image/svg+xml,...` is a double-prefix obfuscation trick.
        // It doesn't match any of our allowed data:image/* prefixes.
        let out =
            sanitize_panicking(r#"<svg><a href="data:data:image/svg+xml,%3Csvg onload='alert(1)'%3E">x</a></svg>"#);
        assert!(!out.contains("data:data:"), "got: {out}");
    }

    #[test]
    fn mixed_case_xlink_href_blocked() {
        // `XLinK:HrEf` exercises both colon-separated attribute name lowercasing
        // and URL validation on the result.
        let out = sanitize_panicking(
            r#"<svg xmlns:xlink="http://www.w3.org/1999/xlink"><a XLinK:HrEf="javascript:alert(1)">x</a></svg>"#,
        );
        assert!(!out.contains("javascript"), "got: {out}");
    }

    #[test]
    fn data_uri_blocked() {
        let out = sanitize_panicking(r#"<svg><a href="data:text/html,<script>alert(1)</script>">x</a></svg>"#);
        assert!(!out.contains("data:"), "got: {out}");
    }

    // ── DOCTYPE / entity expansion ─────────────────────────────────────────────

    #[test]
    fn doctype_dropped() {
        let input = r#"<?xml version="1.0"?><!DOCTYPE svg><svg/>"#;
        let out = sanitize_panicking(input);
        assert!(!out.contains("DOCTYPE"), "got: {out}");
    }

    /// Billion Laughs: the parser must not expand the entities and must not hang.
    #[test]
    fn billion_laughs_safe() {
        let input = r#"<!DOCTYPE svg [
            <!ENTITY lol "lol">
            <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
            <!ENTITY lol2 "&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;">
        ]>
        <svg><text>&lol2;</text></svg>"#;

        // The only thing being tested here is that the function returns promptly
        // without hanging or panicking — the Billion Laughs attack is a DoS via
        // exponential entity expansion, not a data corruption issue. Whether the
        // result is Ok or Err depends on how quick-xml handles undeclared entity
        // references in the body after the DOCTYPE is dropped, which is an
        // implementation detail we don't control or want to pin down.
        let _ = sanitize(input);
    }

    // ── CDATA ──────────────────────────────────────────────────────────────────

    #[test]
    fn cdata_converted_to_text() {
        let out = sanitize_panicking(r#"<svg><text><![CDATA[Hello <world>]]></text></svg>"#);
        assert!(!out.contains("CDATA"), "got: {out}");
        assert!(out.contains("Hello"), "got: {out}");
        // The < must be escaped in the output
        assert!(!out.contains("<world>"), "got: {out}");
        assert!(out.contains("&lt;world&gt;"), "got: {out}");
    }

    // ── Comments and PIs ───────────────────────────────────────────────────────

    #[test]
    fn comments_dropped() {
        let out = sanitize_panicking(r#"<svg><!-- secret --><rect/></svg>"#);
        assert!(!out.contains("secret"), "got: {out}");
        assert!(out.contains("<rect"), "got: {out}");
    }

    #[test]
    fn processing_instructions_dropped() {
        let out = sanitize_panicking(r#"<?xml-stylesheet href="evil.css"?><svg><rect/></svg>"#);
        assert!(!out.contains("xml-stylesheet"), "got: {out}");
        assert!(!out.contains("evil"), "got: {out}");
    }

    // ── Valid content passes through ───────────────────────────────────────────

    #[test]
    fn valid_svg_passes_through() {
        let input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="40" fill="red"/>
  <rect x="10" y="10" width="80" height="80" stroke="blue" fill="none"/>
</svg>"#;
        let out = sanitize_panicking(input);
        assert!(out.contains(r#"<circle"#), "got: {out}");
        assert!(out.contains(r#"fill="red""#), "got: {out}");
        assert!(out.contains(r#"<rect"#), "got: {out}");
    }

    #[test]
    fn aria_attrs_pass_through() {
        let out = sanitize_panicking(r#"<svg><rect aria-label="box" aria-hidden="true"/></svg>"#);
        assert!(out.contains("aria-label"), "got: {out}");
        assert!(out.contains("aria-hidden"), "got: {out}");
    }

    #[test]
    fn data_attrs_pass_through() {
        let out = sanitize_panicking(r#"<svg><rect data-id="42" data-color="blue"/></svg>"#);
        assert!(out.contains("data-id"), "got: {out}");
        assert!(out.contains("data-color"), "got: {out}");
    }
}
