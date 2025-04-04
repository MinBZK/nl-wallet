# sd_jwt_vc_metadata

This crate contains an implementation of SD-JWT VC Type Metadata according to
[this specification](https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html).

## Normalization

When evaluating a chain of type metadata documents that extend one another, the
following normalization rules are applied iteratively in order to use the
metadata for converting the attributes of an attestation to a display
representation:

-   The `display` metadata entries are merged based on the `lang` field:
    -   Entries with the same language are overwritten entirely by the entry
        contained in the extending document. This means that the individual
        properties for an entry are not merged.
    -   For merged entries, the order of the extended document is maintained.
        New entries from extending documents are appended to these, based on the
        order from that extending document.
-   The `claim` metadata entries are merged based on the `path` field:
    -   The paths of the claims of an extending document must be a superset of
        the paths in an extended document. This requirement exists so that the
        order of claims can be fully overridden by an extending document.
    -   The `display` property of a claim is merged according to the same rules
        as the `display` property of the metadata document itself, see above.
    -   The `sd` property can only ever be changed by the extending document by
        becoming more restrictive, i.e. a values of `always` can be changed to
        either `never` or `always`. Any other change constitutes an error.
    -   The `svg_id` of the extending document takes presence. This means that,
        if an extended document has `svg_id` set, but the extending document
        does not, normalization will effectively remove it.

## Resource integrity

When creating or editing type metadata JSON documents that extend other
documents, the resource integrity string for the extended document can be
calculated using the following command:

```bash
echo sha256-$(cat <extended JSON file> | openssl sha256 -binary | openssl base64 -e -A)
```
