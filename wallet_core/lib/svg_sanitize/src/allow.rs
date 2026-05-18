use std::collections::HashSet;
use std::sync::OnceLock;

use quick_xml::events::attributes::Attribute;

pub struct LowerCaseString(String);

impl LowerCaseString {
    pub fn new<T: Into<String>>(s: T) -> Self {
        Self(s.into().to_ascii_lowercase())
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

/// Checks if a tag is allowed.
///
/// Deliberately omits:
/// - `script`, `style`             — code execution / unfiltered CSS
/// - `use`                         — can cause DoS via recursive expansion
/// - `animate`, `set`              — can modify href/other attrs dynamically (XSS vector)
/// - `foreignobject` and `feimage` — embeds arbitrary HTML
/// - `animatecolor`, `tref`, `font`, `glyph`, `glyphref`,
///   `altglyph`, `altglyphdef`, `altglyphitem`, `hkern`, `vkern`
///                                 — deprecated in SVG 2.0; no modern use
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
    .contains(tag.get())
}

/// Checks if an attribute is allowed.
///
/// `style` is included but its value is NOT sanitized (CSS is deferred).
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
            // Style (CSS content unfiltered — known gap, future work)
            "style",
            // Layout and transform
            "alignment-baseline",
            "baseline-shift",
            "transform",
            "transform-origin",
            "viewbox",
            "preserveaspectratio",
            // Identity and linking
            "id",
            "class",
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
    .contains(attr.get())
}

/// Returns true if this attribute's value must be validated as a URL.
pub fn is_url_attr(attr: &LowerCaseString) -> bool {
    matches!(attr.get(), "href" | "xlink:href" | "src")
}

/// An string that has been URL-unescaped, so that checking if a URL is safe
/// will catch encoded payloads like `java&#115;cript:`.
pub struct UrlUnescapedString(String);

impl UrlUnescapedString {
    pub fn new(attr: &Attribute<'_>) -> Result<Self, quick_xml::Error> {
        Ok(Self(attr.unescape_value()?.to_string()))
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

/// Conservative URL allowlist. Permits fragment refs, https, and inert image
/// data URIs. Blocks http://, javascript:, data:image/svg+xml, /, file://,
/// vbscript:, etc.
///
/// `data:image/svg+xml` is intentionally excluded: an SVG-in-SVG data URI
/// executes as a same-origin document in most browsers and is an XSS vector.
pub fn is_safe_url(value: &UrlUnescapedString) -> bool {
    let value = value.get();

    if value.is_empty() {
        return true;
    }

    if value.starts_with('#') || value.starts_with("https://") {
        return true;
    }

    // Allow inert raster image data URIs.
    [
        "data:image/png;",
        "data:image/jpeg;",
        "data:image/gif;",
        "data:image/webp;",
        "data:image/avif",
    ]
    .iter()
    .any(|format| value.starts_with(format))
}

/// Allows `aria-*` and `data-*` attributes through regardless of the static set.
pub fn is_allowed_by_prefix(attr: &LowerCaseString) -> bool {
    attr.get().starts_with("aria-") || attr.get().starts_with("data-")
}
