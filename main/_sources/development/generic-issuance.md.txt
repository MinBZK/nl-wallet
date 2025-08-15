# Generic Issuance

The NL Wallet uses [SD-JWT VC Type Metadata](https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#name-sd-jwt-vc-type-metadata) documents for validating both [SD-JWT](https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html) _and_ [ISO mdoc](https://www.iso.org/obp/ui/en/#iso:std:iso-iec:18013:-5:ed-1:v1:en) formatted attestations. The following paragraphs describe how this works.

## SD-JWT
The SD-JWT VC Type Metadata is applied to SD-JWT formatted attestations as described in the SD-JWT VC specification.

## mdoc
For mdoc, no formal metadata specification exists. Instead of developing a custom metadata scheme for mdoc attestations, the NL Wallet uses the SD-JWT VC Type Metadata with some adaptations to describe the contents of an mdoc formatted attestation.

The relevant differences between mdocs and SD-JWT formatted attestations are:

1. Mdoc documents use the [CBOR](https://datatracker.ietf.org/doc/html/rfc8949) serialization format, whereas SD-JWT documents use JSON.
2. Contrary to SD-JWT, in mdocs arbitrarily deep nesting of claims is not supported: In mdocs, the attribute values are put in attribute groups called the "namespace". In other words, mdocs always have a nesting of exactly one level deep.

The remainder of this document explains how the NL Wallet deals with these differences.

### 1. Serialize the mdoc to JSON for JSON Schema
SD-JWT VC Type Metadata documents may contain a [JSON Schema](https://json-schema.org/draft/2020-12/json-schema-core) document to enable verifying the structure and format of the attestation and the attributes within it, ensuring that the attestatation has certain fields and attributes of certain JSON data types. After deserializing the CBOR-formatted attestation, the NL Wallet serializes it to JSON as described in Section [6.1 of the CBOR specification](https://datatracker.ietf.org/doc/html/rfc8949#name-converting-from-cbor-to-jso) to enable validating it against the JSON Schema contained in the SD-JWT VC Type Metadata document.

### 2. Convert namespace structure to nested SD-JWT structure
Before verifying the mdoc against the SD-JWT VC Type Metadata, the mdoc needs to be transformed to mimic the structure that an SD-JWT formatted attestation based on the same Type Metadata would have. This is done using the `path` array in the `claims` objects in the Type Metadata (see the example below). Iterating over the attributes in the `claims` object in the Type Metadata, for each attribute:

 - The final element of the `path` array is the attribute name.
 - The mdoc namespace is computed by (1) concatenating the remaining elements of the `path` array with a dot `.` and (2) prefixing that with the attestation type.
 - Given this namespace and attribute name, lookup the attribute value in the mdoc.
 - Use the `path` array to construct the location of the attribute inside the nested JSON structure, and place the attribute value there.

### Example

#### SD-JWT VC Type Metadata used for validation

In the example below, the following SD-JWT VC Type Metadata is used.

```json
{
  "vct": "com.example.address",
  // ... display data of the attestation here
  "claims": [
    {
      "path": [ "city" ],
      "display": [ /* ... */ ],
      "sd": "always"
    },
    {
      "path": [ "street" ],
      "display": [ /* ... */ ],
      "sd": "always"
    },
    {
      "path": [ "house", "number" ],
      "display": [ /* ... */ ],
      "sd": "always"
    },
    {
      "path": [ "house", "letter" ],
      "display": [ /* ... */ ],
      "sd": "always"
    }
  ],
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "properties": {
      "vct":           { "type": "string" },
      "vct#integrity": { "type": "string" },
      "city":          { "type": "string" },
      "street":        { "type": "string" },
      "house": {
        "type": "object",
        "properties": {
          "number":    { "type": "number" },
          "letter":    { "type": "string"}
        },
        "minProperties": 1
      }
    },
    "required": [ "vct", "vct#integrity", "cnf", "iat", "exp", "city", "street", "house" ]
  }
}
```

#### Mdoc formatted document snippet
Suppose that the mdoc looks as follows. In this JSON snippet, the object keys in the root object denote the mdoc namespaces and the elements under those are the attributes.

```json
// doctype/vct is "com.example.address"
{
    "com.example.address": {
        "city": "The Capital",
        "street": "Main St."
    },
    "com.example.address.house": {
        "number": 1,
        "letter": "A"
    }
}
```

#### Transformed snippet used for validation against Type Metadata
The JSON structure is constructed using the claims paths in the Type Metadata (see above). This results in the following reconstructed SD-JWT claimset.

```json
// doctype/vct is "com.example.address"
{
  "city": "The Capital",
  "street": "Main St.",
  "house": {
      "number": 1,
      "letter": "A"
   }
}
```
