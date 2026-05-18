use std::collections::HashSet;
use std::sync::OnceLock;

use derive_more::AsRef;
use quick_xml::events::attributes::Attribute;

#[derive(Debug, Clone, AsRef)]
pub struct LowerCaseString(String);

impl LowerCaseString {
    pub fn new<T: Into<String>>(s: T) -> Self {
        Self(s.into().to_ascii_lowercase())
    }
}

/// Checks if a tag is allowed.
///
/// Deliberately omits:
/// - `script`, `style`             — code execution / unfiltered CSS
/// - `use`                         — can cause DoS via recursive expansion
/// - `animate`, `set`              — can modify href/other attrs dynamically (XSS vector)
/// - `foreignobject`, `feimage`    — embeds arbitrary HTML
/// - `animatecolor`, `tref`, `font`, `glyph`, `glyphref`, `altglyph`, `altglyphdef`, `altglyphitem`, `hkern`, `vkern` —
///   deprecated in SVG 2.0; no modern use
pub fn is_allowed_tag(tag: &LowerCaseString) -> bool {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();

    SET.get_or_init(|| {
        HashSet::from([
            // SVG structural
            "svg",
            "defs",
            "g",
            "view",
            "switch",
            "metadata",
            // Shapes
            "circle",
            "ellipse",
            "line",
            "path",
            "polygon",
            "polyline",
            "rect",
            // Text
            "text",
            "tspan",
            "textpath",
            "title",
            "desc",
            // Links and images
            "a",
            "image",
            // Gradients and patterns
            "lineargradient",
            "radialgradient",
            "stop",
            "pattern",
            // Clip / mask / filter containers
            "filter",
            "marker",
            "mask",
            "clippath",
            // Safe animation subset (no attribute-mutating elements)
            "animatemotion",
            "animatetransform",
            "mpath",
            // Filter primitives — stored lowercase; input is lowercased before lookup
            "feblend",
            "fecolormatrix",
            "fecomponenttransfer",
            "fecomposite",
            "feconvolvematrix",
            "fediffuselighting",
            "fedisplacementmap",
            "fedistantlight",
            "feflood",
            "fefunca",
            "fefuncb",
            "fefuncg",
            "fefuncr",
            "fegaussianblur",
            "femerge",
            "femergenode",
            "femorphology",
            "feoffset",
            "fepointlight",
            "fespecularlighting",
            "fespotlight",
            "fetile",
            "feturbulence",
        ])
    })
    .contains(tag.as_ref().as_str())
}

/// Checks if an attribute is allowed.
///
/// `style` and `class` are excluded: the renderer does not support CSS, so
/// both are useless and would otherwise be an unfiltered injection surface.
/// Event handler attributes (`onclick`, `onload`, etc.) are absent.
pub fn is_allowed_attr(attr: &LowerCaseString) -> bool {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();

    SET.get_or_init(|| {
        HashSet::from([
            // Geometry and shape
            "cx",
            "cy",
            "d",
            "dx",
            "dy",
            "fx",
            "fy",
            "height",
            "width",
            "r",
            "rx",
            "ry",
            "x",
            "x1",
            "x2",
            "y",
            "y1",
            "y2",
            "z",
            "points",
            "path",
            "pathlength",
            // Presentation
            "clip",
            "clip-path",
            "clip-rule",
            "color",
            "color-interpolation",
            "color-interpolation-filters",
            "color-profile",
            "color-rendering",
            "direction",
            "display",
            "dominant-baseline",
            "fill",
            "fill-opacity",
            "fill-rule",
            "filter",
            "flood-color",
            "flood-opacity",
            "font-family",
            "font-size",
            "font-size-adjust",
            "font-stretch",
            "font-style",
            "font-variant",
            "font-weight",
            "image-rendering",
            "letter-spacing",
            "lighting-color",
            "marker-end",
            "marker-mid",
            "marker-start",
            "mask",
            "opacity",
            "overflow",
            "paint-order",
            "shape-rendering",
            "stop-color",
            "stop-opacity",
            "stroke",
            "stroke-dasharray",
            "stroke-dashoffset",
            "stroke-linecap",
            "stroke-linejoin",
            "stroke-miterlimit",
            "stroke-opacity",
            "stroke-width",
            "text-anchor",
            "text-decoration",
            "text-rendering",
            "vector-effect",
            "visibility",
            "word-spacing",
            "writing-mode",
            // Layout and transform
            "alignment-baseline",
            "baseline-shift",
            "transform",
            "transform-origin",
            "viewbox",
            "preserveaspectratio",
            // Identity and linking
            "id",
            "name",
            "lang",
            "tabindex",
            "href",
            "src",
            "version",
            // Gradient / pattern / filter attributes
            "gradientunits",
            "gradienttransform",
            "patterncontentunits",
            "patterntransform",
            "patternunits",
            "maskcontentunits",
            "maskunits",
            "filterunits",
            "primitiveunits",
            "spreadmethod",
            "offset",
            "local",
            // Filter primitive attributes
            "in",
            "in2",
            "result",
            "mode",
            "operator",
            "order",
            "kernelmatrix",
            "kernelunitlength",
            "bias",
            "divisor",
            "targetx",
            "targety",
            "edgemode",
            "preservealpha",
            "scale",
            "xchannelselector",
            "ychannelselector",
            "stddeviation",
            "azimuth",
            "elevation",
            "diffuseconstant",
            "surfacescale",
            "specularconstant",
            "specularexponent",
            "limitingconeangle",
            "pointsatx",
            "pointsaty",
            "pointsatz",
            "basefrequency",
            "numoctaves",
            "seed",
            "stitchtiles",
            "type",
            "tablevalues",
            "slope",
            "intercept",
            "amplitude",
            "exponent",
            "k",
            "k1",
            "k2",
            "k3",
            "k4",
            "radius",
            "refx",
            "refy",
            "markerheight",
            "markerunits",
            "markerwidth",
            "orient",
            "lengthadjust",
            "textlength",
            "startoffset",
            // Clippath
            "clippathunits",
            // Text presentation (deprecated in SVG 2 but valid on text/tspan)
            "kerning",
            // Symbol / view
            "zoomandpan",
            // Animation (safe subset)
            "attributename",
            "attributetype",
            "begin",
            "dur",
            "end",
            "min",
            "max",
            "repeatcount",
            "repeatdur",
            "restart",
            "by",
            "from",
            "to",
            "values",
            "calcmode",
            "keytimes",
            "keysplines",
            "keypoints",
            "additive",
            "accumulate",
            "rotate",
            "origin",
            // Switch
            "systemlanguage",
            "requiredextensions",
            // Media / accessibility
            "media",
            "wrap",
            "orientation",
            // XML namespace attributes (with colon — matched as literal strings)
            "xmlns",
            "xmlns:xlink",
            "xlink:href",
            "xlink:title",
            "xlink:type",
            "xlink:actuate",
            "xlink:show",
            "xlink:role",
            "xlink:arcrole",
            "xml:id",
            "xml:space",
            "xml:lang",
            // xml:base is intentionally excluded: it rebases relative URI resolution,
            // allowing `href="#x"` to resolve against a javascript: base URL.
            // Modern browsers dropped xml:base support (Firefox 66+, 2019), so there
            // is no legitimate use case that justifies the risk.
        ])
    })
    .contains(attr.as_ref().as_str())
}

/// Returns true if this attribute's value must be validated as a URL.
pub fn is_url_attr(attr: &LowerCaseString) -> bool {
    ["href", "xlink:href", "src"].contains(&attr.as_ref().as_str())
}

/// Returns true if this attribute's value may contain CSS `url()` references
/// that could trigger outbound network requests.
pub fn is_url_func_attr(attr: &LowerCaseString) -> bool {
    [
        "clip-path",
        "mask",
        "filter",
        "fill",
        "stroke",
        "marker-start",
        "marker-mid",
        "marker-end",
        "color-profile",
    ]
    .contains(&attr.as_ref().as_str())
}

/// Returns true if every `url(...)` in `value` references a local fragment (i.e. starts with `#`).
/// A value may contain multiple `url()` references (e.g. `filter="url(#blur) url(#contrast)"`);
/// all of them must pass. Values containing no `url()` references are always safe.
/// Handles optional surrounding quotes and inner whitespace per the CSS `url()` syntax.
///
/// Also rejects values containing alternative CSS image-loading functions (`image()`,
/// `image-set()`, `cross-fade()`, `src()`) that can reference external URLs but whose
/// argument syntax differs from `url()`.
pub fn has_safe_url_func(value: &LowerCaseString) -> bool {
    let v = value.as_ref().as_str();

    if ["image(", "image-set(", "-webkit-image-set(", "cross-fade(", "src("]
        .iter()
        .any(|f| v.contains(f))
    {
        return false;
    }

    // Walk every url() in the value; each one must reference a local fragment.
    // If they don't, we consider them unsafe and return false.
    let mut rest = v;
    while let Some(idx) = rest.find("url(") {
        rest = &rest[idx + 4..];
        let Some(end) = rest.find(')') else {
            return false; // malformed url() with no closing parent
        };
        let inner = rest[..end].trim().trim_matches(|c| c == '\'' || c == '"').trim();
        if !inner.starts_with('#') {
            return false;
        }
        rest = &rest[end + 1..];
    }

    true
}

/// An string that has been URL-unescaped, so that checking if a URL is safe
/// will catch encoded payloads like `java&#115;cript:`.
#[derive(Debug, Clone, AsRef)]
pub struct UrlUnescapedString(String);

impl UrlUnescapedString {
    pub fn new(attr: &Attribute<'_>) -> Result<Self, quick_xml::Error> {
        Ok(Self(attr.unescape_value()?.to_string()))
    }
}

/// Conservative URL allowlist. Permits only fragment refs and inert raster
/// image data URIs. Blocks all external URLs (http://, https://, //),
/// javascript:, data:image/svg+xml, /, file://, vbscript:, etc.
///
/// External URLs are blocked entirely so that SVGs cannot trigger outbound
/// network requests that would allow tracking users or leaking credentials.
///
/// `data:image/svg+xml` is intentionally excluded: an SVG-in-SVG data URI
/// executes as a same-origin document in most browsers and is an XSS vector.
pub fn is_safe_url(value: &UrlUnescapedString) -> bool {
    let value = value.as_ref();

    if value.is_empty() {
        return true;
    }

    if value.starts_with('#') {
        return true;
    }

    [
        "data:image/png;",
        "data:image/jpeg;",
        "data:image/gif;",
        "data:image/webp;",
        "data:image/avif",
    ]
    .iter()
    .any(|format| value.to_ascii_lowercase().starts_with(format))
}

/// Allows `aria-*` and `data-*` attributes through regardless of the static set.
pub fn is_allowed_by_prefix(attr: &LowerCaseString) -> bool {
    attr.as_ref().starts_with("aria-") || attr.as_ref().starts_with("data-")
}
