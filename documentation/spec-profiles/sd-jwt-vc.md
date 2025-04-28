# SD-JWT VC Profile for the NL Wallet, VV and OV
The following paragraphs describe a profile of the SD-JWT VC implementation within the NL Wallet, VV and OV.
This profile should be read alongside the original specification, [SD-JWT VC Draft 08](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-sd-jwt-vc-08) [SD-JWT-VC].

[[_TOC_]]

## SD-JWT VC Type Metadata

### Type Metadata
- `name`
    - REQUIRED (OPTIONAL in [SD-JWT-VC])
- `claims`
    - REQUIRED (OPTIONAL in [SD-JWT-VC]): See [Claims Metadata](#claims-metadata)
- `display`
    - REQUIRED (OPTIONAL in [SD-JWT-VC]). See [Display Metadata](#display-metadata). At least one Display Metadata containing `simple` rendering MUST be present.
- `schema`
    - REQUIRED (OPTIONAL in [SD-JWT-VC]). An embedded JSON Schema document describing the structure of the Verifiable Credential.
- `schema_uri`
    - NOT SUPPORTED (OPTIONAL in [SD-JWT-VC]). Schema information MUST be embedded using `schema`.

### Display Metadata
- `lang` 
    - REQUIRED, must be unique for every object in the `display` array.
- `summary` (not present in [SD-JWT-VC])
    - OPTIONAL. Contains a summary of the credential (see [Credential Summary](#credential-summary)).
- `rendering`
    - OPTIONAL. Only `simple` rendering is supported.


#### Credential Summary

The optional `summary` field of the Display Metadata provides a summary of the credential for the end user, and may contain `svg_id` identifiers from the [Claim metadata](#claim-metadata) as placeholders for credential attributes.
The NL Wallet will render this field as a subtitle of the card, with the `svg_id` identifiers replaced by the appropriate attribute values.

Example:
```json
"summary" : "PID card for {{firstname}} {{lastname}}"
```

If the values of the `firstname` and `lastname` are `John` and `Doe` respectively, then in the NL Wallet this will be rendered as `PID card for John Doe`.

#### Simple Rendering
The NL Wallet supports `simple` rendering as specified, except the `uri` of [Logo Metadata](#logo-metadata). 

Rendering using `svg_templates` is not supported. 


##### Logo metadata
- `uri`
    - REQUIRED. MUST use `data` URI scheme (no external links).
    Example: `"data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vc..."`



### Claims Metadata

- All leaf values in the credential MUST be selectable by a claim in the `claims` section (excluding reserved claims). Reserved claims: `vct`, `cnf`, `iss`, `exp`, `iat`, `sub`, `status`, `attestation_qualification`.
- Any credential containing values that are not selectable by one of the `claims` will be rejected.


_Supported Data types_

The NL Wallet supports rendering of claims that select any the following types:
- JSON types: `boolean`, `number` and `string`
- To format date, time or date-time values from `string`-types, the NL Wallet uses the `format` property from the JSON Schema, which can be one of
`date`, `time` or `datetime`. If no format or another format is present, `string` is assumed without further processing. The NL Wallet does *not* support claims that select a JSON `array` or `object` value. 
- Type Metadata MUST only contain claim paths that select the supported types above. Any Type Metadata that contains claims that select a non-supported type, will be rejected.


#### Claim Metadata

- `display`
    - REQUIRED (OPTIONAL in [SD-JWT-VC]). At least one Claim Display Metadata object MUST be present in the array.
- `svg_id`
    - OPTIONAL. Only used for [Credential Summary](#credential-summary). SVG rendering is not supported.

#### Claim Display Metadata
- `lang`
    - REQUIRED. MUST be unique for every object in the `display` array.



