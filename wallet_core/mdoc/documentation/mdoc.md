# Introduction to ISO mdoc

This document introduces the key concepts from the mdoc ISO standards (ISO 18013-5 and -7), as well as the major data structures.

## General notions

### mdoc

The mdoc (mobile document) is the main citizen of the ISO 18013-5 and -7 standards, which define the mdoc data types as well as protocols to use and receive them.
An mdoc contains data (attributes) about the holder, the public key of the holder, and the issuer signature over those.
It is granted by a trusted issuer to the holder, which receives and stores it.
Later, the holder can selectively disclose attributes from the mdoc to a RP (Relying Party).

"Mdoc" is a term specific to these ISO standards. In other contexts, this concept is generally called "credential" or "certificate" instead.
Below, we stick with the term mdoc.

### Doctypes

A doctype relates to an mdoc roughly the way a class relates to a class instance.
The doctype is a string that identifies the type of an mdoc, which in turn determines the attribute names that can occur in the mdoc.
An example is `"org.iso.18013.5.1.mDL"`, defined by the ISO 18013-5 standard.
Table 5 in section 7.2.1 of the standard defines the attribute names that may or must be present in mdocs of that
doctype.

### Mdoc data model

An attribute of an mdoc consists of the following data:
- `digestID`: ID for the attribute, incrementing integer starting at 0;
- `random`: random bytes, a sort of salt for selective disclosure (see below);
- `elementIdentifier`: attribute name;
- `elementValue`: attribute value.

These are not included directly in the issuer-signed part of the mdoc; instead for each attribute the digest (hash) of this data structure is.
Thus, the holder of the mdoc keeps track of this datastructure alongside the issuer-signed part of the mdoc.
When using an mdoc to disclose attributes to a RP, the holder can hide (i.e. not disclose) an attribute by simply not sending this datastructure to the RP.
The presence of the `random` bytes prevents the RP from trying to "reverse" the digest by trying if likely attribute values result in the digests in the mdoc.

In an mdoc, attributes are grouped within namespaces. The structure of an mdoc is schematically as follows.
- private key
- attributes: identifier, name, value, random bytes
- issuer-signed COSE with MSO (`MobileSecurityObject`) as payload, which contains a.o.:
  - doctype
  - signing and expiry dates
  - mdoc public key (corresponding to private key)
  - one or more namespaces
    - per namespace, the digests of the attributes inside the namesapce

When disclosing an attribute, the holder sends the COSE with the MSO inside it to the RP, along with the attributes that it wants to disclose.
In addition, it uses the private key of the mdoc (whose corresponding public key is included in the MSO) to sign a particular data structure that is unique to each session. This proves to the RP that the holder indeed has possession of the mdoc and it prevents replay attacks.

### Data encoding

#### CBOR

All data structures in ISO 18013-5 and -7 are encoded using [CBOR](https://cbor.io/) ([RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html)), which is similar to JSON except that it is binary and has more features.
Like JSON, data structures are generally maps with names and values (unlike ASN.1, the other major binary encoding format).
It being binary, it can avoid base64 for binary data so that in that case it is more efficient than JSON, which helps for data like digital signatures.
However, this also makes it not human-readable. Instead it is generally displayed to humans hex-encoded.

<https://cbor.me/> is a simple tool to convert hex-encoded CBOR to a JSON-like diagnostic human readable notation.
This website does not prettyprint its output, which is cumbersome with data structures as large as they tend to be in the ISO standards.
The Rust tool [`prettycbor`](TODO) has the same output but prettyprinted.

The ISO 18013-5 standard has examples of most data structures defined in the standard, shown as hex-encoded CBOR in appendix D.
All of the examples below come from that appendix, using `prettycbor` (and they are also included in `src/tests/examples.rs`).

#### CDDL

Associated to CBOR is the CDDL standard ([RFC 8610](https://www.rfc-editor.org/rfc/rfc8610)), with which one can define CBOR (and JSON) data structures.
The ISO standards use CDDL to define its data structures.
An example of CDDL showing some of its features is the following:

<!--- I know I know, this really isn't python. But no markdown renderer that I know comes with CDDL syntax highlighting,
      and the python syntax highlighter makes this look sort of okay. --->
```python
ExampleDataStructure = {
   "title": tstr,                  # a field called `title` containing a string
   ? "data": bstr,                 # an optional field called `data` containing binary data
   "input": Inputs,                # a field called `input` of type `Inputs` defined below
}

Inputs = {
    "vars": { + tstr => any }      # a map of at least one entry; keys are strings and values may be any type
    "entry": [ * tstr ]            # an array of any number of elements (0 or more)
}
```

Fields of type `bstr` contain arbitrary bytes. By contrast, a `tstr` represents a UTF8 string.
The following CBOR (in diagnostic notation) would satisfy this definition:

<!--- Again, using jsonc here as a good second best. --->
```jsonc
{
    "title": "foo",
    "data": h'DEADBEEF', // binary data shown hex-encoded
    "input": {
        "vars": {
            "qux": 1,
            "quux": true
        },
        "entry": [
            "bar",
            "baz"
        ]
    }
}
```

#### COSE

COSE ([RFC 8152](https://www.rfc-editor.org/rfc/rfc8152.html)) is for CBOR what JOSE is for JSON.
In the ISO standards, it generally refers to the CBOR-equivalent of a JWT: a signed data structure, using ECDSA or HMAC 256/256.
The ISO standards use COSE for everything that needs to be signed, including mdocs themselves.

Like a JWT, a COSE contains a header, payload, and a signature over those two.
The COSE standard has the following CDDL data structure definition of a COSE that uses digital signatures.

```python
COSE_Sign1 = [
    Headers,
    payload : bstr / nil,  # payload is either binary data or absent
    signature : bstr
]
```

The payload is not always present in the COSE; sometimes it is transmitted out of band.

The COSE standard also includes a data structure definition called `COSE_Key` for cryptographic keys, such as an ECDSA public key.
The ISO standards use this for all keys occuring in the mdoc data structures.

### Verification

An mdoc is signed by its issuer through the signature over the MSO (Mobile Security Object, see below), in COSE form.

The public key corresponding to the issuer's private key is encoded in an X509 certificate, and included in a header of the mdoc COSE.
This issuer X509 certificate is signed in turn by the IACA, the Issuer Authority Certificate Authority, which thus acts as the root of trust.
An RP may trust one or more IACAs.

To verify an mdoc (disclosure), the RP thus needs to perform the following steps:

1. Parse the mdoc, and take the issuer X509 certificate from the COSE header of the mdoc.
2. Validate this certificate against the IACA certificate(s) that the RP trusts.
3. Using the public key from the issuer certificate, verify the mdoc.
4. For each attribute, verify that the digest of the attribute (including its index and `random`) is contained within
   the mdoc (and thus effectively signed by the issuer).

This just verifies the mdoc data structure and attributes. To establish liveness, the holder additionally needs to
prove possession of the private key corresponding to the mdoc's public key, by signing a challenge of the RP:

5. Parse the public key contained in the mdoc.
6. Using the mdoc's public key, verify the holder's signature over the challenge.

## Data structures

This section describes and shows examples of the major datatypes of ISO 18013-5.
In the descriptions, we use a CDDL-like notation to indicate the type of the fields.
The examples are shown in prettyprinted CBOR-diagnostic notation, produced by running `prettycbor` on the hexadecimal examples from appendix D of ISO 18013-5.

These data structures can also be studied as follows.

- The examples shown below may also be seen in the Rust `dbg!` format, by running the tests of this project using `--nocapture`, for example as follows:
  ```
  cargo test -- iso_examples_disclosure --nocapture
  ```
  This results in output that is a lot more descriptive than the CBOR-diagnostic notation below, at the cost of a lot more verbosity.
- Using the rustdocs of the implementing structs in the [`iso`](../src/iso/) subcrate, for example by generating the
  crate's documentation as follows:
  ```
  cargo doc --open
  ```

The ISO standard shows the following patterns.
- CBOR data types can be arbitrarily nested, in the sense that one data type can contain another as shown in the example above in the CBOR section.
  However, in the ISO standard sometimes an inner data structure is not included directly inside an outer data structure.
  Instead the outer data structure contains a byte sequence (`bstr`), which is to contain inner data structure in CBOR-encoded form.
  This is done in cases where the exact encoding matters, such as when the data acts as input to a hash function or as the payload of a digital signature.
  An example from the standard is as follows:
  ```
  IssuerNameSpaces = {
      + NameSpace => [ + IssuerSignedItemBytes ]
  }

  IssuerSignedItemBytes = #6.24(bstr .cbor IssuerSignedItem)

  IssuerSignedItem = {
      "digestID": uint,
      "random": bstr,
      "elementIdentifier": DataElementIdentifier,
      "elementValue": DataElementValue
  }
  ```
  This example defines `IssuerSignedItemBytes` as a `bstr` that contains CBOR data (indicated by `#6.24`) of type `IssuerSignedItem`.
  When this happens, the name of the byte-version of the datastructure always has a `Bytes` suffix as seen here.

  In the examples below, we will for clarity however ignore the distinction between data structures and their `Bytes` suffixed equivalents.
- In the ISO standards, most data structures are defined as maps having fixed keys. However, some data structures are instead defined as follows:
    - as arrays (i.e. without keys), for example `DeviceAuthentication` (see below);
    - as maps having incrementing integer keys (for example `DeviceEngagement`),

  Why the standard sometimes uses these variants is not clear.
  This implementation always uses Rust structs with field names for clarity, and then converts those to the appropriate form during encoding using helper data types with custom (de)serializers.

### IssuerSigned

All mdoc data except its private key, containing the data signed by the issuer during issuance, as well as some (during disclosure) or all of the attributes of the mdoc (in case the data structure is part of an mdoc held by the holder).

`IssuerSigned`
- `nameSpaces` (`+ tstr => [+ IssuerSignedItem]`): maps namespace to multiple attributes including their IDs and randoms
- `issuerAuth` (COSE over `MobileSecurityObject`): data signed by the issuer during issuance
    - COSE signature created by issuer during issuance
    - COSE header which (a.o.) contains the certificate with which the COSE is signed. This certificate itself is signed by a CA which the holder and RP must trust.
    - COSE payload: `MobileSecurityObject`
        - `version` (`tstr`): version of this data structure
        - `digestAlgorithm` (`tstr`): message digest algorithm used (in practice, always "SHA256")
        - `valueDigests` (`ValueDigests`): digests of all data elements per namespace
        - `deviceKeyInfo` (`DeviceKeyInfo`): public key of the mdoc
        - `docType` (`tstr`): the doctype of the mdoc
        - `validityInfo` (`ValidityInfo`): issuance and expiry date of the mdoc

`IssuerSignedItem`: an attribute including its ID and random
- `digestID` (`int`): ID for the attribute, incrementing, starts at 0
- `random` (`bstr`): random bytes, acting as a salt, for selective disclosure
- `elementIdentifier` (`tstr`): attribute name
- `elementValue` (`any`): attribute value

#### Example

This example comes from a disclosure, as only attributes 0 and 7 are included; the absence of the rest means that they are not disclosed. (Holders use the same data structure for their mdocs, but then all attributes are always present.)

Below, the `24(<< ... >>)` syntax means that the data inside the `<< >>` is encoded as a byte sequence containing CBOR. The `h'DEADBEEF'` syntax is a byte sequence literal shown hex-encoded.

```js
"issuerSigned": {
    "nameSpaces": {
        "org.iso.18013.5.1": [
            24(<<{
                "digestID": 0,
                "random": h'8798645B20EA200E19FFABAC92624BEE6AEC63ACEEDECFB1B80077D22BFC20E9',
                "elementIdentifier": "family_name",
                "elementValue": "Doe"
            }>>),
            24(<<{
                "digestID": 7,
                "random": h'26052A42E5880557A806C1459AF3FB7EB505D3781566329D0B604B845B5F9E68',
                "elementIdentifier": "document_number",
                "elementValue": "123456789"
            }>>),
        ]
    },
    "issuerAuth":
        <<{
            1: -7
        }>>,
        {
            // X509 certificate of the issuer that signs this MSO
            33: h'308201EF30820195A00302010202143C4416EED784F3B413E48F56F075ABFA6D87EB84300A06082A8648CE3D04030230233114301206035504030C0B75746F7069612069616361310B3009060355040613025553301E170D3230313030313030303030305A170D3231313030313030303030305A30213112301006035504030C0975746F706961206473310B30090603550406130255533059301306072A8648CE3D020106082A8648CE3D03010703420004ACE7AB7340E5D9648C5A72A9A6F56745C7AAD436A03A43EFEA77B5FA7B88F0197D57D8983E1B37D3A539F4D588365E38CBBF5B94D68C547B5BC8731DCD2F146BA381A83081A5301E0603551D120417301581136578616D706C65406578616D706C652E636F6D301C0603551D1F041530133011A00FA00D820B6578616D706C652E636F6D301D0603551D0E0416041414E29017A6C35621FFC7A686B7B72DB06CD12351301F0603551D2304183016801454FA2383A04C28E0D930792261C80C4881D2C00B300E0603551D0F0101FF04040302078030150603551D250101FF040B3009060728818C5D050102300A06082A8648CE3D040302034800304502210097717AB9016740C8D7BCDAA494A62C053BBDECCE1383C1ACA72AD08DBC04CBB202203BAD859C13A63C6D1AD67D814D43E2425CAF90D422422C04A8EE0304C0D3A68D'
        },
        <<24(<<{
            "version": "1.0",
            "digestAlgorithm": "SHA-256",
            "valueDigests": {
                // Digests of the attributes per namespace
                "org.iso.18013.5.1": {
                    0: h'75167333B47B6C2BFB86ECCC1F438CF57AF055371AC55E1E359E20F254ADCEBF',
                    1: h'67E539D6139EBD131AEF441B445645DD831B2B375B390CA5EF6279B205ED4571',
                    2: h'3394372DDB78053F36D5D869780E61EDA313D44A392092AD8E0527A2FBFE55AE',
                    3: h'2E35AD3C4E514BB67B1A9DB51CE74E4CB9B7146E41AC52DAC9CE86B8613DB555',
                    4: h'EA5C3304BB7C4A8DCB51C4C13B65264F845541341342093CCA786E058FAC2D59',
                    5: h'FAE487F68B7A0E87A749774E56E9E1DC3A8EC7B77E490D21F0E1D3475661AA1D',
                    6: h'7D83E507AE77DB815DE4D803B88555D0511D894C897439F5774056416A1C7533',
                    7: h'F0549A145F1CF75CBEEFFA881D4857DD438D627CF32174B1731C4C38E12CA936',
                    8: h'B68C8AFCB2AAF7C581411D2877DEF155BE2EB121A42BC9BA5B7312377E068F66',
                    9: h'0B3587D1DD0C2A07A35BFB120D99A0ABFB5DF56865BB7FA15CC8B56A66DF6E0C',
                    10: h'C98A170CF36E11ABB724E98A75A5343DFA2B6ED3DF2ECFBB8EF2EE55DD41C881',
                    11: h'B57DD036782F7B14C6A30FAAAAE6CCD5054CE88BDFA51A016BA75EDA1EDEA948',
                    12: h'651F8736B18480FE252A03224EA087B5D10CA5485146C67C74AC4EC3112D4C3A'
                },
                "org.iso.18013.5.1.US": {
                    0: h'D80B83D25173C484C5640610FF1A31C949C1D934BF4CF7F18D5223B15DD4F21C',
                    1: h'4D80E1E2E4FB246D97895427CE7000BB59BB24C8CD003ECF94BF35BBD2917E34',
                    2: h'8B331F3B685BCA372E85351A25C9484AB7AFCDF0D2233105511F778D98C2F544',
                    3: h'C343AF1BD1690715439161ABA73702C474ABF992B20C9FB55C36A336EBE01A87'
                }
            },
            "deviceKeyInfo": {
                // ECDSA public key of the mdoc in COSE_Key format
                "deviceKey": {
                    1: 2,
                    -1: 1,
                    -2: h'96313D6C63E24E3372742BFDB1A33BA2C897DCD68AB8C753E4FBD48DCA6B7F9A',
                    -3: h'1FB3269EDD418857DE1B39A4E4A44B92FA484CAA722C228288F01D0C03A2C3D6'
                }
            },
            "docType": "org.iso.18013.5.1.mDL",
            "validityInfo": {
                "signed": 0("2020-10-01T13:30:02Z"),
                "validFrom": 0("2020-10-01T13:30:02Z"),
                "validUntil": 0("2021-10-01T13:30:02Z")
            }
        }>>)>>,
        // Issuer signature
        h'59E64205DF1E2F708DD6DB0847AED79FC7C0201D80FA55BADCAF2E1BCF5902E1E5A62E4832044B890AD85AA53F129134775D733754D7CB7A413766AEFF13CB2E'
    ]
}
```

### DeviceRequest

This message is sent after session establishment by the RP to the holder, to indicate which attributes out of which mdocs it wants.

`DeviceRequest`
- `version` (`tstr`): version of this data structure
- `docRequests` (`+ DocRequest`): the requested attributes per doctype, including RP authentication
    - `itemsRequest` (`ItemsRequest`): the requested attributes per doctype, including RP authentication
        - `docType` (`tstr`): doctype of the requested mdoc
        - `nameSpaces` (`+ NameSpace => {+ tstr => bool}`): per namespace, the requested attributes including a boolean indicating the intent to retain
        - `requestInfo` (`* tstr => any`): freeform extra information
    - `readerAuth` (`? ReaderAuth`): optional COSE authenticating this `DocRequest`

#### Example

```js
{
    "version": "1.0",
    "docRequests": [
        {
            "itemsRequest": 24(<<{
                "docType": "org.iso.18013.5.1.mDL",
                "nameSpaces": {
                    "org.iso.18013.5.1": {
                        "family_name": true,
                        "document_number": true,
                        "driving_privileges": true,
                        "issue_date": true,
                        "expiry_date": true,
                        "portrait": false
                    }
                }
            }>>),
            "readerAuth": [
                <<{
                    1: -7
                }>>,
                {
                    33: h'308201B330820158A00302010202147552715F6ADD323D4934A1BA175DC945755D8B50300A06082A8648CE3D04030230163114301206035504030C0B72656164657220726F6F74301E170D3230313030313030303030305A170D3233313233313030303030305A3011310F300D06035504030C067265616465723059301306072A8648CE3D020106082A8648CE3D03010703420004F8912EE0F912B6BE683BA2FA0121B2630E601B2B628DFF3B44F6394EAA9ABDBCC2149D29D6FF1A3E091135177E5C3D9C57F3BF839761EED02C64DD82AE1D3BBFA38188308185301C0603551D1F041530133011A00FA00D820B6578616D706C652E636F6D301D0603551D0E04160414F2DFC4ACAFC5F30B464FADA20BFCD533AF5E07F5301F0603551D23041830168014CFB7A881BAEA5F32B6FB91CC29590C50DFAC416E300E0603551D0F0101FF04040302078030150603551D250101FF040B3009060728818C5D050106300A06082A8648CE3D0403020349003046022100FB9EA3B686FD7EA2F0234858FF8328B4EFEF6A1EF71EC4AAE4E307206F9214930221009B94F0D739DFA84CCA29EFED529DD4838ACFD8B6BEE212DC6320C46FEB839A35'
                },
                null,
                h'1F3400069063C189138BDCD2F631427C589424113FC9EC26CEBCACACFCDB9695D28E99953BECABC4E30AB4EFACC839A81F9159933D192527EE91B449BB7F80BF'
            ]
        }
    ]
}
```

### DeviceResponse

This message is sent by the holder to the RP, in response to a `DeviceResponse`. It contains all disclosed attributes.

`DeviceResponse`
- `version` (`tstr`): version of this data structure
- `documents` (`+ Document`): mdoc disclosures
    - `docType` (`tstr): doctype of the mdoc
    - `issuerSigned` (`IssuerSigned`): (subset of) attributes and issuer signed data, see above
    - `deviceSigned` (`DeviceSigned`): data signed by the holder of the mdoc during disclosure
        - `nameSpaces` (`+ tstr => DeviceSignedItem`): map of namespace to self-asserted attributes
        - `deviceAuth`: COSE (`Sign1` or `Mac0`):
            - COSE signature created by holder during disclosure
            - COSE payload: `DeviceAuthentication` payload (not included), an array containing the following:
                - `"DeviceAuthentication"`: hardcoded string,
                - `SessionTranscript`: a transcript of the entire session so far
                - `DocType`: doctype of the mdoc
                - `DeviceNameSpacesBytes`: self-asserted attributes identical to the one above
    - `errors` (`? Errors`): errors per attribute that happened during disclosure of the mdoc
- `documentErrors` (`+ DocumentError`): errors per mdoc that happened during disclosure
- `status` (`uint`): describes the status of the disclosure (0 for ok, other values for certain errors)

#### Example

```js
{
    "version": "1.0",
    "documents": [
        {
            "docType": "org.iso.18013.5.1.mDL",
            "issuerSigned": { ... } // see `IssuerSigned` above,
            "deviceSigned": {
                "nameSpaces": 24(<<{}>>), // this example contains no self-asserted attributes
                "deviceAuth": {
                    "deviceMac": [
                        <<{
                            1: 5 // COSE protected header indicating the HMAC 256/256 algorithm
                        }>>,
                        {},
                        null,
                        h'E99521A85AD7891B806A07F8B5388A332D92C189A7BF293EE1F543405AE6824D' // COSE MAC
                    ]
                }
            }
        }
    ],
    "status": 0
}
```
