use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

#[derive(Debug, Default)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!("IdentityHasher only supports writing numbers")
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    fn write_usize(&mut self, i: usize) {
        self.0 = u64::try_from(i).expect("usize too large to fit in u64");
    }
}

type SparseStatusVec = HashMap<usize, StatusType, BuildHasherDefault<IdentityHasher>>;

/// A Status List is a data structure that contains the statuses of many Referenced Tokens represented by one or
/// multiple bits.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-4>
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StatusList {
    sparse: SparseStatusVec,
    len: usize,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Eq))]
pub struct PackedStatusList {
    /// The amount of bits used to describe the status of each Referenced Token within this Status List.
    bits: Bits,

    lst: Vec<u8>,
}

/// JSON representation of a [`PackedStatusList`], as used by the JWT profile, in which `lst` is a base64url-encoded
/// string. Use with `#[serde(with = "JsonPackedStatusList")]`.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-4.2>
#[derive(Serialize, Deserialize)]
#[serde(remote = "PackedStatusList")]
pub(crate) struct JsonPackedStatusList {
    bits: Bits,

    #[serde(with = "zlib_base64")]
    lst: Vec<u8>,
}

/// CBOR representation of a [`PackedStatusList`], as used by the CWT profile, in which `lst` is a byte string. Use
/// with `#[serde(with = "CborPackedStatusList")]`.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-4.3>
#[derive(Serialize, Deserialize)]
#[serde(remote = "PackedStatusList")]
pub(crate) struct CborPackedStatusList {
    bits: Bits,

    #[serde(with = "zlib_bytes")]
    lst: Vec<u8>,
}

#[cfg(test)]
impl PartialEq for PackedStatusList {
    fn eq(&self, other: &Self) -> bool {
        self.unpack() == other.unpack()
    }
}

mod zlib {
    use std::io::prelude::*;

    use flate2::Compression;
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;

    pub fn compress<W: Write>(writer: W, bytes: &[u8]) -> std::io::Result<W> {
        // Implementations are RECOMMENDED to use the highest compression level available.
        let mut e = ZlibEncoder::new(writer, Compression::best());
        e.write_all(bytes)?;
        e.finish()
    }

    pub fn decompress(reader: impl Read, capacity: usize) -> std::io::Result<Vec<u8>> {
        let mut decompressed = Vec::with_capacity(capacity);
        ZlibDecoder::new(reader).read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}

mod zlib_base64 {
    use base64::prelude::*;
    use serde::Deserialize;
    use serde::Serialize;

    use super::zlib;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Best gzip compression is limited by the algorithm to 1032, rounded to a nice two power
        let buf = Vec::with_capacity(bytes.len() / 1024);
        let writer = base64::write::EncoderWriter::new(buf, &BASE64_URL_SAFE_NO_PAD);

        let mut writer = zlib::compress(writer, bytes).map_err(serde::ser::Error::custom)?;

        let encoded = writer.finish().map_err(serde::ser::Error::custom)?;
        let encoded = String::from_utf8(encoded).expect("base64 encoded string should be valid utf-8");
        encoded.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;

        let reader = base64::read::DecoderReader::new(encoded.as_bytes(), &BASE64_URL_SAFE_NO_PAD);
        zlib::decompress(reader, encoded.len()).map_err(serde::de::Error::custom)
    }
}

mod zlib_bytes {
    use serde::Deserialize;
    use serde_bytes::ByteBuf;

    use super::zlib;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Best gzip compression is limited by the algorithm to 1032, rounded to a nice two power
        let buf = Vec::with_capacity(bytes.len() / 1024);
        let compressed = zlib::compress(buf, bytes).map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&compressed)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let compressed = ByteBuf::deserialize(deserializer)?;
        zlib::decompress(compressed.as_slice(), compressed.len()).map_err(serde::de::Error::custom)
    }
}

impl StatusList {
    pub fn new(len: usize) -> Self {
        StatusList {
            sparse: HashMap::default(),
            len,
        }
    }

    pub fn new_aligned(len: usize, bits: Bits) -> Self {
        Self::new(bits.aligned_len(len))
    }

    pub fn get(&self, k: usize) -> StatusType {
        self.sparse.get(&k).copied().unwrap_or_default()
    }

    pub fn insert(&mut self, k: usize, v: StatusType) -> Option<StatusType> {
        if k > self.len {
            panic!(
                "status list was created with size {}, cannot insert at position {k}",
                self.len
            )
        }

        self.sparse.insert(k, v)
    }

    pub fn pack(self) -> PackedStatusList {
        let bits = self.sparse.values().map(|s| s.bits()).max().unwrap_or_default(); // empty list
        let size = bits.packed_len(self.len);
        let lst = self.sparse.iter().fold(vec![0; size], |mut acc, (index, status)| {
            let idx = bits.packed_index(*index);
            acc[idx] |= Into::<u8>::into(*status) << bits.shift_for_index(*index);
            acc
        });

        PackedStatusList { bits, lst }
    }
}

impl PackedStatusList {
    pub fn new(len: usize) -> PackedStatusList {
        let bits = Bits::One;
        PackedStatusList {
            bits,
            lst: vec![0; bits.packed_len(len)],
        }
    }

    pub fn all_invalid(len: usize) -> PackedStatusList {
        let bits = Bits::One;
        PackedStatusList {
            bits,
            lst: vec![0xff; bits.packed_len(len)],
        }
    }

    pub fn bits(&self) -> &Bits {
        &self.bits
    }

    pub(crate) fn get(&self, index: usize) -> Option<StatusType> {
        let byte = self.lst.get(self.bits.packed_index(index))?;
        let status = (byte >> self.bits.shift_for_index(index)) & self.bits.mask();
        Some(StatusType::from(status))
    }

    pub fn unpack(&self) -> StatusList {
        let len = self.bits.unpacked_len(self.lst.len());
        let sparse: SparseStatusVec = (0..len)
            .filter_map(|index| {
                let status = self.get(index)?;
                (status != StatusType::Valid).then_some((index, status))
            })
            .collect();

        StatusList { sparse, len }
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.lst.is_empty()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bits {
    #[default]
    One,
    Two,
    Four,
    Eight,
}

impl Serialize for Bits {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.size())
    }
}

impl<'de> Deserialize<'de> for Bits {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match u8::deserialize(deserializer)? {
            1 => Ok(Bits::One),
            2 => Ok(Bits::Two),
            4 => Ok(Bits::Four),
            8 => Ok(Bits::Eight),
            n => Err(serde::de::Error::custom(format!(
                "invalid value: {n}, expected one of: 1, 2, 4, 8"
            ))),
        }
    }
}

impl Bits {
    #[inline]
    fn size(self) -> u8 {
        1 << self as u8
    }

    #[inline]
    fn statuses_per_byte(self) -> usize {
        8 >> self as usize
    }

    #[inline]
    fn mask(self) -> u8 {
        ((1 << u16::from(self.size())) - 1) as u8
    }

    #[inline]
    fn shift_for_index(self, index: usize) -> usize {
        let remainder = index & (self.statuses_per_byte() - 1);
        remainder * self.size() as usize
    }

    #[inline]
    fn inverse_exponent(self) -> usize {
        3 - self as usize
    }

    #[inline]
    fn packed_index(self, index: usize) -> usize {
        index >> self.inverse_exponent()
    }

    #[inline]
    fn unpacked_len(self, size: usize) -> usize {
        size << self.inverse_exponent()
    }

    #[inline]
    fn packed_len(self, len: usize) -> usize {
        self.aligned_len(len) >> self.inverse_exponent()
    }

    #[inline]
    fn aligned_len(self, len: usize) -> usize {
        let align = self.statuses_per_byte();
        (len + (align - 1)) & !(align - 1)
    }
}

/// A status describes the state, mode, condition or stage of an entity that is represented by the Referenced Token.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-21.html#section-7>
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    /// The status of the Referenced Token is valid, correct or legal.
    #[default]
    Valid, // 0x00

    /// The status of the Referenced Token is revoked, annulled, taken back, recalled or cancelled.
    Invalid, // 0x01

    /// The status of the Referenced Token is temporarily invalid, hanging, debarred from privilege. This state is
    /// usually temporary.
    Suspended, // 0x02

    /// The Status Type value 0x03 and Status Type values in the range 0x0C until 0x0F are permanently reserved as
    /// application specific. The processing of Status Types using these values is application specific.
    ApplicationSpecific(u8),

    /// All other Status Type values are reserved for future registration.
    Reserved(u8),
}

impl From<u8> for StatusType {
    fn from(value: u8) -> Self {
        match value {
            0 => StatusType::Valid,
            1 => StatusType::Invalid,
            2 => StatusType::Suspended,
            3 => StatusType::ApplicationSpecific(3),
            0x0C..=0x0F => StatusType::ApplicationSpecific(value),
            _ => StatusType::Reserved(value),
        }
    }
}

impl From<StatusType> for u8 {
    fn from(value: StatusType) -> Self {
        match value {
            StatusType::Valid => 0,
            StatusType::Invalid => 1,
            StatusType::Suspended => 2,
            StatusType::ApplicationSpecific(i) | StatusType::Reserved(i) => i,
        }
    }
}

impl StatusType {
    fn bits(self) -> Bits {
        match self {
            StatusType::Valid | StatusType::Invalid => Bits::One,
            StatusType::Suspended | StatusType::ApplicationSpecific(3) => Bits::Two,
            StatusType::ApplicationSpecific(i) | StatusType::Reserved(i) if i <= 0x0F => Bits::Four,
            StatusType::Reserved(_) => Bits::Eight,
            _ => panic!("invalid status type"),
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::sync::LazyLock;

    use regex::regex;
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    // Parse the status list examples as they are listed in the spec
    fn parse_status_list(input: &str) -> StatusList {
        let sparse = regex!(r"status\[(\d+)]\s*=\s*(\d+)").captures_iter(input).fold(
            SparseStatusVec::default(),
            |mut acc, cap| {
                let index = cap.get(1).unwrap().as_str().parse::<usize>().unwrap();
                let value = cap.get(2).unwrap().as_str().parse::<u8>().unwrap();

                if value != 0 {
                    acc.insert(index, StatusType::from(value));
                }
                acc
            },
        );

        let len = sparse.keys().max().unwrap().to_owned() + 1;
        StatusList { len, sparse }
    }

    pub static EXAMPLE_STATUS_LIST_ONE: LazyLock<StatusList> =
        LazyLock::new(|| parse_status_list(include_str!("../examples/spec/example-status-list-1.txt")));
    static EXAMPLE_STATUS_LIST_TWO: LazyLock<StatusList> =
        LazyLock::new(|| parse_status_list(include_str!("../examples/spec/example-status-list-2.txt")));
    static ONE_BIT_STATUS_LIST: LazyLock<StatusList> =
        LazyLock::new(|| parse_status_list(include_str!("../examples/spec/1-bit-status-list.txt")));
    static TWO_BIT_STATUS_LIST: LazyLock<StatusList> =
        LazyLock::new(|| parse_status_list(include_str!("../examples/spec/2-bit-status-list.txt")));
    static FOUR_BIT_STATUS_LIST: LazyLock<StatusList> =
        LazyLock::new(|| parse_status_list(include_str!("../examples/spec/4-bit-status-list.txt")));
    static EIGHT_BIT_STATUS_LIST: LazyLock<StatusList> =
        LazyLock::new(|| parse_status_list(include_str!("../examples/spec/8-bit-status-list.txt")));

    #[rstest]
    #[case(EXAMPLE_STATUS_LIST_ONE.to_owned(), json!({
            "bits": 1,
            "lst": "eNrbuRgAAhcBXQ",
        })
    )]
    #[case(EXAMPLE_STATUS_LIST_TWO.to_owned(), json!({
            "bits": 2,
            "lst": "eNo76fITAAPfAgc"
        })
    )]
    fn test_status_list_serialization(#[case] list: StatusList, #[case] expected: serde_json::Value) {
        let compressed = JsonPackedStatusList::serialize(&list.pack(), serde_json::value::Serializer).unwrap();
        assert_eq!(compressed, expected);
    }

    #[rstest]
    #[case(ONE_BIT_STATUS_LIST.to_owned(), Bits::One)]
    #[case(TWO_BIT_STATUS_LIST.to_owned(), Bits::Two)]
    #[case(FOUR_BIT_STATUS_LIST.to_owned(), Bits::Four)]
    #[case(EIGHT_BIT_STATUS_LIST.to_owned(), Bits::Eight)]
    fn test_status_list_serialization_bits(#[case] list: StatusList, #[case] expected: Bits) {
        let compressed = JsonPackedStatusList::serialize(&list.pack(), serde_json::value::Serializer).unwrap();
        assert_eq!(compressed["bits"].as_u64().unwrap(), u64::from(expected.size()));
    }

    #[test]
    fn test_status_list_cbor_serialization_matches_spec_example() {
        #[rustfmt::skip]
        let expected: Vec<u8> = vec![
            0xa2, // map(2)
            0x64, 0x62, 0x69, 0x74, 0x73, // text(4) "bits"
            0x01, // uint(1)
            0x63, 0x6c, 0x73, 0x74, // text(3) "lst"
            0x4a, // bytes(10)
            0x78, 0xda, 0xdb, 0xb9, 0x18, 0x00, 0x02, 0x17, 0x01, 0x5d,
        ];

        #[derive(Serialize, Deserialize)]
        struct Cbor(#[serde(with = "CborPackedStatusList")] PackedStatusList);

        let packed = EXAMPLE_STATUS_LIST_ONE.to_owned().pack();
        let mut encoded = Vec::new();
        ciborium::ser::into_writer(&Cbor(packed.clone()), &mut encoded).unwrap();
        assert_eq!(encoded, expected);

        let Cbor(decoded) = ciborium::de::from_reader(expected.as_slice()).unwrap();
        assert_eq!(decoded, packed);
    }

    #[rstest]
    #[case(json!({
            "bits": 1,
            "lst": "eNrbuRgAAhcBXQ",
        }),
        EXAMPLE_STATUS_LIST_ONE.to_owned()
    )]
    #[case(json!({
            "bits": 2,
            "lst": "eNo76fITAAPfAgc"
        }),
        EXAMPLE_STATUS_LIST_TWO.to_owned()
    )]
    #[case(json!({
            "bits": 1,
            "lst":
                "eNrt3AENwCAMAEGogklACtKQPg9LugC9k_ACvreiogEAAKkeCQAAAAAAAAAAAAAAAAAAAIBylgQAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAXG9IAAAAAAAAAPwsJAAAAAAAAAAAAAAAvhsSAAAAAAAAAAAA7KpLAAAAAAAAAAAAAAAAAAAAAJ\
                 sLCQAAAAAAAAAAADjelAAAAAAAAAAAKjDMAQAAAACAZC8L2AEb"
        }),
        ONE_BIT_STATUS_LIST.to_owned()
    )]
    #[case(json!({
            "bits": 2,
            "lst":
                "eNrt2zENACEQAEEuoaBABP5VIO01fCjIHTMStt9ovGVIAAAAAABAbiEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEB5W\
                 wIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAID0ugQAAAAAAAAAAAAAAAAAQG12SgAAAAAAAAAAAAAAAAAAAAAAAAAAAOCSIQEAAAAAAAAAAAAAAAAAAAAAAAD8ExIAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAwJEuAQAAAAAAAAAAAAAAAAAAAAAAAMB9SwIAAAAAAAAAAAAAAAAAAACoYUoAAAAA\
                 AAAAAAAAAEBqH81gAQw"
        }),
        TWO_BIT_STATUS_LIST.to_owned()
    )]
    #[case(json!({
            "bits": 4,
            "lst":
                "eNrt0EENgDAQADAIHwImkIIEJEwCUpCEBBQRHOy35Li1EjoOQGabAgAAAAAAAAAAAAAAAAAAACC1SQEAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABADrsCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADoxaEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIIoCgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAACArpwKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGhqVkAzlwIAAAAAiGVRAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAABx3AoAgLpVAQAAAAAAAAAAAAAAwM89rwMAAAAAAAAAAAjsA9xMBMA"
        }),
        FOUR_BIT_STATUS_LIST.to_owned()
    )]
    #[case(json!({
            "bits": 8,
            "lst":
                "eNrt0WOQM2kYhtGsbdu2bdu2bdu2bdu2bdu2jVnU1my-SWYm6U5enFPVf7ue97orFYAo7CQBAACQuuckAABStqUEAAAAAAAAtN6wE\
                 gAE71QJAAAAAIrwhwQAAAAAAdtAAgAAAAAAACLwkAQAAAAAAAAAAACUaFcJAACAeJwkAQAAAAAAAABQvL4kAAAAWmJwCQAAAAAAAA\
                 jAwBIAAAB06ywJoDKQBARpfgkAAAAAAAAAAAAAAAAAAACo50sJAAAAAAAAAOiRcSQAAAAAgAJNKgEAAG23mgQAAAAAAECw3pUAQve\
                 gBAAAAAAAAADduE4CAAAAyjSvBAAQiw8koHjvSABAb-wlARCONyVoxtMSZOd0CQAAAOjWDRKQmLckAAAAAACysLYEQGcnSAAAAAAQ\
                 ooUlAABI15kSAIH5RAIgLB9LABC4_SUgGZNIAABAmM6RoLbTJIASzCIBAEAhfpcAAAAAAABquk8CAAAAAAAAaJl9SvvzBOICAFWmk\
                 IBgfSgBAAAANOgrCQAAAAAAAADStK8EAAC03gASAAAAAAAAAADFWFUCAAAAMjOaBEADHpYAQjCIBADduFwCAAAAAGitMSSI3BUSAE\
                 COHpAA6IHrJQAAAAAAsjeVBAAAKRpVAorWvwQAAAAAAAAAkKRtJAAAAAAAgCbcLAF0bXUJAAAAoF02kYDg7CYBAAAAAEB6NpQAAAA\
                 AAAAAAAAAAEr1uQQAAF06VgIAAAAAAAAAqDaeBAAQqgMkAAAAAABogQMlAAAAAAAa87MEAAAQiwslAAAAAAAAAAAAAAAAMrOyBAAA\
                 iekv-hcsY0Sgne6QAAAAAAAgaUtJAAAAAAAAAAAAAAAAAAAAAAAAAADwt-07vjVkAAAAgDy8KgFAUEaSAAAAAJL3vgQAWdhcAgAAo\
                 BHDSUDo1pQAAACI2o4SAABZm14CALoyuwQAAPznGQkgZwdLAAAQukclAAAAAAAAAAAAgKbMKgEAAAAAAAAAAAAAAAAAAECftpYAAA\
                 AAAAAAAAAACnaXBAAAAADk7iMJAAAAAAAAAABqe00CAnGbBBG4TAIAgFDdKgFAXCaWAAAAAAAAAAAAAAAAAKAJQwR72XbGAQAAAKA\
                 hh0sAAAAAAABQgO8kAAAAAAAAAAAAACAaM0kAAAC5W0QCAIJ3mAQAxGwxCQAA6nhSAsjZBRIAANEbWQIAAAAAaJE3JACAwA0qAUBI\
                 VpKAlphbAiAPp0iQnKEkAAAAAAAgBP1KAAAAdOl4CQAAAAAAAPjLZBIAAG10RtrPm8_CAEBMTpYAAAAAAIjQYBL8z5QSAAAAAEDYP\
                 pUAACAsj0gAAADQkHMlAAjHDxIA0Lg9JQAAgHDsLQEAAABAQS6WAAAAgLjNFs2l_RgLAIAEfCEBlGZZCQAAaIHjJACgtlskAAAozb\
                 0SAAAAVFtfAgAAAAAAAAAAAAAAAAAAAAAAAKDDtxIAAAAAVZaTAKB5W0kAANCAsSUgJ0tL0GqHSNBbL0gAZflRAgCARG0kQXNmlgC\
                 ABiwkAQAAAEB25pIAAAAAAAAAAAAAoFh9SwAAAAAAADWNmOSrpjFsEoaRgDKcF9Q1dxsEAAAAAAAAAAAAAAAAgPZ6SQIAAAAAAAAA\
                 gChMLgEAAAAAAAAAqZlQAsK2qQQAAAAAAAD06XUJAAAAqG9bCQAAgLD9IgEAAAAAAAAAAAAAAAAAAEBNe0gAAAAAAAAAAEBPHSEBA\
                 AAAlOZtCYA4fS8B0GFRCQAo0gISAOTgNwmC840EAAAAAAAAAAAAAAAAAAAAUJydJfjXPBIAAAAAAAAAAAAAAABk6WwJAAAAAAAAAA\
                 AAAAAAqG8UCQAAgPpOlAAAIA83SQAANWwc9HUjGAgAAAAAAACAusaSAAAAAAAAAAAAAAAAAAAAAAAAAAAAqHKVBACQjxklAAAAAAA\
                 AAKBHxpQAAAAAACBME0lAdlaUAACyt7sEAAAA0Nl0EgAAAAAAAAAAAABA-8wgAQAAAAAAAKU4SgKgUtlBAgAAAAAAAAAAgMCMLwEE\
                 51kJICdzSgCJGl2CsE0tAQAA0L11JQAAAAAAAAjUOhIAAAAAAAAAAAAAAGTqeQkAAAAAAAAAAAAAKM8SEjTrJwkAAAAAAACocqQEU\
                 LgVJAAAACjDUxJUKgtKAAAAqbpRAgCA0n0mAQAAAABAGzwmAUCTLpUAAAAAAAAAAEjZNRIAAAAAAAAAAAAAAAAAAAAA8I-vJaAlhp\
                 QAAAAAAHrvzjJ-OqCuuVlLAojP8BJAr70sQZVDJYAgXS0BAAAAAAAAAAAAtMnyEgAAAAAAFONKCQAAAAAAAADorc0kAAAAAAAAgDq\
                 OlgAAAAAAAAAAAADIwv0SAAAAAAAAAAAAAADBuV0CIFVDSwAAAABAAI6RAAAAAGIwrQSEZAsJAABouRclAAAAAKDDrxIAAAA0bkkJ\
                 gFiMKwEAAAAAAHQyhwRk7h4JAAAAAAAAAAAgatdKAACUYj0JAAAAAAAAAAAAQnORBLTFJRIAAAAAkIaDJAAAAJryngQAAAAAAAAAA\
                 AA98oQEAAAAAAAAAEC2zpcgWY9LQKL2kwAgGK9IAAAAAPHaRQIAAAAAAAAAAADIxyoSAAAAAAAAAAAAAADQFotLAECz_gQ1PX-B"
        }),
        EIGHT_BIT_STATUS_LIST.to_owned()
    )]
    fn test_status_list_deserialization(#[case] value: serde_json::Value, #[case] expected: StatusList) {
        let packed = JsonPackedStatusList::deserialize(value).unwrap();

        let status_list = packed.unpack();

        assert_eq!(status_list.sparse, expected.sparse);
        // Since the example input only specifies non-valid entries, there can be more entries in the deserialized list
        assert!(status_list.len >= expected.len);
    }

    #[rstest]
    #[case(0, StatusType::Invalid)]
    #[case(1, StatusType::Valid)]
    #[case(1993, StatusType::Suspended)]
    #[case(35460, StatusType::ApplicationSpecific(3))]
    fn test_get(#[case] index: usize, #[case] expected: StatusType) {
        let packed = FOUR_BIT_STATUS_LIST.to_owned().pack();
        assert_eq!(packed.get(index), Some(expected));
    }

    #[test]
    fn test_get_out_of_bounds() {
        let packed = PackedStatusList::new(8);
        assert_eq!(packed.get(8), None);
    }

    #[test]
    fn test_packed_status_list_new() {
        let packed = PackedStatusList::new(15);
        let unpacked = packed.unpack();
        assert_eq!(unpacked.len, 16);
        assert_eq!(unpacked.sparse, HashMap::default());
    }

    #[test]
    fn test_packed_status_all_invalid() {
        let packed = PackedStatusList::all_invalid(15);
        let unpacked = packed.unpack();
        assert_eq!(unpacked.len, 16);
        let expected_sparse = (0..16).map(|i| (i, StatusType::Invalid)).collect::<HashMap<_, _, _>>();
        assert_eq!(unpacked.sparse, expected_sparse);
    }
}
