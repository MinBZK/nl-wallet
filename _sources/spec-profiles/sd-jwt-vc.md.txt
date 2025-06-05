# SD-JWT VC Profile for the NL Wallet, VV and OV

## Introduction
The NL Wallet, VV, and OV implement the [SD-JWT VC Draft 08](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-sd-jwt-vc-08) [SD-JWT-VC] specification, with *additional constraints and customizations* outlined in the profile below.

Our goal is to contribute to a fully interoperable EUDI framework that delivers maximum value to users and organizations. Achieving this requires consensus on standards and implementation profiles. To contribute to that consensus, this profile reflects our current implementation and design preferences, based on the experience from designing, developing and testing the NL Wallet. Note that it should not be read as the final specifications of our product, or the broader Dutch ecosystem. Nor is it an official position of the Dutch government.

Considerations that underpin the choices made in this profile are described in [Profile considerations](#profile-considerations).

Ideally, the profile is temporary and our final product implements the same specifications as all others within the EU and Dutch framework. Until that time, attestation providers (issuers) and relying parties wishing to integrate with the current development version of the NL Wallet, VV, and OV must adhere to this profile.

## SD-JWT VC Type Metadata

### Type Metadata
- `extends`
    - OPTIONAL, A URI of another type that this type extends. See [Extending Types](#extending-types). `extends#integrity` is required when referring to an extended type.
- `extends#integrity`
    - OPTIONAL, MUST be present when `extends` is present.
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
"summary" : "Person data for {{firstname}} {{lastname}}"
```

If the values of the `firstname` and `lastname` are `John` and `Doe` respectively, then in the NL Wallet this will be rendered as `Person data for John Doe`.

Example display of a credential in compact card form, using the `summary` from the example above:
```
+----------------------------+
| PID card                   | // `name` from Display Metadata
+----------------------------+
| Person data for John Doe   | // Credential Summary (`summary`) from Display Metadata.
|                            | // using {firstname} and {lastname} from claims
+----------------------------+ 
```


#### Simple Rendering
The NL Wallet supports `simple` rendering as specified, except the `uri` of [Logo Metadata](#logo-metadata). 

Rendering using `svg_templates` is not supported. 


##### Logo metadata
- `uri`
    - REQUIRED. MUST use `data` URI scheme as defined in [RFC 2397](https://datatracker.ietf.org/doc/html/rfc2397) (no external links).
    Example: `"data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vc..."`
        The following mime-types are supported: `image/jpeg`, `image/png`, `image/svg+xml`. 


### Claims Metadata

- All leaf values in the credential MUST be selectable by a claim in the `claims` section (excluding reserved claims). Reserved claims: `vct`, `cnf`, `iss`, `exp`, `iat`, `sub`, `status`, `attestation_qualification`.
- Any credential containing values that are not selectable by one of the `claims` will be rejected.


_Supported Data types_

The NL Wallet supports rendering of claims that select any of the following types:
- JSON types: `boolean`, `number` and `string`
- To format date, time or date-time values from `string`-types, the NL Wallet uses the `format` property from the JSON Schema, which can be one of
`date`, `time` or `datetime`. If no format or another format is present, `string` is assumed without further processing. The NL Wallet does *not* (yet) support claims that select a JSON `array` or `object` value. 
- Type Metadata MUST only contain claim paths that select the supported types above. Any Type Metadata that contains claims that select a non-supported type, will be rejected.


#### Claim Metadata

- `display`
    - REQUIRED (OPTIONAL in [SD-JWT-VC]). At least one Claim Display Metadata object MUST be present in the array.
- `svg_id`
    - OPTIONAL. Only used for [Credential Summary](#credential-summary). SVG rendering is not supported.

#### Claim Display Metadata
- `lang`
    - REQUIRED. MUST be unique for every object in the `display` array.



### Extending types

Extending types is supported. [SD-JWT-VC] does not explicitly define how extended types should relate to 
base types. 

When processing types that extend one other, the following rules are applied:

-   The `display` metadata entries are merged based on the `lang` field:
    -   Entries within the base document that have the same language as the extending document
        are replaced entirely by the entry contained in the extending document. Individual
        properties for a `display` metadata entry are not merged.
    -   Order: The order of the `display` metadata entries in the base document is maintained.
        New entries from extending documents are appended, based on the
        order from the extending document.
-   The `claim` metadata entries are merged based on the `path` field:
    -   The paths of the claims of an extending document MUST be a superset of
        the paths in the base document. This requirement exists so that the
        order of claims can be fully overridden by an extending document. Any extending document that
        does not have claims matching all paths from the base document will be rejected.
    -   The `display` property of `claim` metadata entries is merged according to the same rules
        as the `display` property of the type metadata document, see above.
    -   The `sd` property can only be overridden if the value is
        `allowed` (which is the default). Once this has been changed to `always`
        or `never`, this constitutes an end state for that property and it can
        no longer be overridden to another value in an extending document.
    -   The `svg_id` of the extending document takes precedence. When
        the base document has `svg_id` set, but the extending document
        does not, the resulting document will not have an `svg_id`.
    -   Type information for all claim paths MUST be defined in the JSON-schema.

## Profile considerations
### C1 - Credential contents must be presentable to the end user

*Issue*

Information necessary for proper presentation of claim data to the end user (consumer of the claim) is not mandatory in [SD-JWT VC], making it difficult for end users to understand the claims they receive and share.

*Motivation*

We aim to use Type Metadata for both the presentation and data definition for a given credential type. From our standpoint, the Type Metadata must contain sufficient information for properly presenting a credential (including all of its attributes and Localized field labels) to an end user. 

*Implication*
- `display` metadata is required for both the Type and Claims metadata.
- `claims` MUST be present in offered [Type Metadata](#type-metadata)
- All values in the credential MUST be referenced by a `claim` in the `claims` section (excluding reserved claims: `vct`, `cnf`, `iss`, `exp`, `iat`, `sub`, `status`, 
`attestation_qualification`). See [Claims Metadata](#claims-metadata).
- Data that is selected by claims must be renderable for the Wallet App. A `claim` path must select a type that can be rendered. See [Claims Metadata](#claims-metadata).
    - `object` types are not supported.
    - `array` types are NOT YET supported.


### C2 - Credential summary (based on contents) for optimized browsing
*Issue*

[SD-JWT VC] does not support defining a textual name or identifier of a specific attestation for the end user, based on the attestation contents, one that sets apart multiple attestations of the same type (e.g. 'Master degree computer science' versus 'Master degree mathematics'). This makes it more difficult to browse stored credentials and find the right one.

*Motivation*

To allow for an optimized browsing UX in a wallet we propose to add a `summary` to the credential [Display Metadata](#display-metadata). The summary can be used by the wallet to create a compact view for the credential. It will give the user a meaningful and recognizable overview of the credential's content, which has proven useful from our current UX explorations. The `summary` may contain placeholders for actual values from the credential, similar to the templating logic for `svg_template` rendering [SD-JWT-VC]. See [Credential Summary](#credential-summary) for more details.


*Implication*
- `summary` as a new OPTIONAL field in the [Display Metadata](#display-metadata).
 - The summary can be used in both rendering modes `simple` and `svg_template` 


### C3 - No support for external resources
*Issue*

[SD-JWT VC] allows for refering to external resources in the following cases:
- Type metadata
    - `schema_uri` 
- Logo metadata
    - `uri`

Downloading resources from external location introduces the following risks:
- Availability: when a resource is not available a fallback is needed
- Performance: when a resource takes time to be delivered to the end user, the user experience may degrade
- Privacy: fetching external resources introduces privacy risks, since user behavior may be tracked by the resource owner.

*Motivation*

To mitigate the risks mentioned we will currenty not allow the use of external resources. Part of the risks might also be mitigated by requirements to the scheme that NL-Wallet is part of. Distributing Type Metadata (and related resources) is not yet addressed in ARF or other frameworks.  

*Implication*
- Type metadata
    - `schema_uri` is NOT supported
- Logo metadata
    - `uri`: only `data` URI scheme is supported 

### C4 - Unambiguously select a locale for displaying credentials
*Issue*

[SD-JWT VC] allows a `lang` attribute to support localization in the Type Metadata specification. 
1. `lang` in [Display Metadata](#display-metadata)
1. `lang` in [Claim Metadata](#claim-metadata)

In [SD-JWT VC] there are no rules defined on what a consuming Wallet should display when multiple items contain the same value for `lang`. This may result to unexpected or undesired behavior.

*Motivation*

To avoid unexpected or undesired behavior we propose an additional restriction on the collections that contain items that are only distinguished by the value of the `lang` attribute.

*Implication*

- `lang` in [Display Metadata](#display-metadata) must be unique, per `rendering` method.
- `lang` in [Claim Metadata](#claim-metadata) must be unique.
