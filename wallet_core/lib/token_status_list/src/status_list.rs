use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;

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
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-status-list>
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StatusList {
    sparse: SparseStatusVec,
    len: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq))]
pub struct PackedStatusList {
    /// The amount of bits used to describe the status of each Referenced Token within this Status List.
    bits: Bits,

    #[serde(with = "zlib_base64")]
    lst: Vec<u8>,
}

#[cfg(test)]
impl PartialEq for PackedStatusList {
    fn eq(&self, other: &Self) -> bool {
        self.unpack() == other.unpack()
    }
}

mod zlib_base64 {
    use std::io::prelude::*;

    use base64::prelude::*;
    use flate2::Compression;
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;
    use serde::Deserialize;
    use serde::Serialize;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Implementations are RECOMMENDED to use the highest compression level available.
        let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
        e.write_all(bytes).map_err(serde::ser::Error::custom)?;
        let compressed = e.finish().map_err(serde::ser::Error::custom)?;

        let encoded = BASE64_URL_SAFE_NO_PAD.encode(compressed);
        encoded.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let decoded = BASE64_URL_SAFE_NO_PAD.decode(s).map_err(serde::de::Error::custom)?;

        let mut d = ZlibDecoder::new(&decoded[..]);
        let mut decompressed = Vec::new();
        d.read_to_end(&mut decompressed).map_err(serde::de::Error::custom)?;

        Ok(decompressed)
    }
}

impl StatusList {
    pub fn new(len: usize) -> Self {
        StatusList {
            sparse: HashMap::default(),
            len,
        }
    }

    pub fn get(&self, k: &usize) -> StatusType {
        self.sparse.get(k).copied().unwrap_or_default()
    }

    pub fn insert(&mut self, k: usize, v: StatusType) -> Option<StatusType> {
        if k > self.len {
            return None; // status lists should not grow dynamically
        }

        self.sparse.insert(k, v)
    }

    pub fn pack(self) -> PackedStatusList {
        let bits = self
            .sparse
            .values()
            .max_by(|a, b| a.bits().cmp(&b.bits()))
            .map(|s| s.bits())
            .unwrap_or_default(); // empty list

        let lst = self.sparse.iter().fold(
            vec![0; (self.len * bits as usize).div_ceil(8)],
            |mut acc, (index, status)| {
                let idx = bits.packed_index(*index);
                acc[idx] |= Into::<u8>::into(*status) << bits.shift_for_index(*index);
                acc
            },
        );

        PackedStatusList { bits, lst }
    }
}

impl PackedStatusList {
    pub fn single_unpack(&self, index: usize) -> StatusType {
        let byte = self.lst[self.bits.packed_index(index)];
        let status = (byte >> self.bits.shift_for_index(index)) & self.bits.mask();
        StatusType::from(status)
    }

    pub fn partial_unpack(&self, indices: &[usize]) -> Vec<StatusType> {
        indices.iter().map(|index| self.single_unpack(*index)).collect()
    }

    pub fn unpack(&self) -> StatusList {
        let len = self.bits.unpacked_len(self.lst.len());
        let sparse: SparseStatusVec = (0..len)
            .filter_map(|index| {
                let status = self.single_unpack(index);
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Bits {
    #[default]
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
}

impl Bits {
    #[inline]
    fn statuses_per_byte(self) -> usize {
        8 / self as usize
    }

    #[inline]
    pub fn mask(self) -> u8 {
        ((1 << self as u16) - 1) as u8
    }

    #[inline]
    pub fn shift_for_index(self, index: usize) -> usize {
        (index % self.statuses_per_byte()) * self as usize
    }

    #[inline]
    pub fn packed_index(self, index: usize) -> usize {
        index / self.statuses_per_byte()
    }

    #[inline]
    pub fn unpacked_len(self, len: usize) -> usize {
        len * self.statuses_per_byte()
    }
}

/// A status describes the state, mode, condition or stage of an entity that is represented by the Referenced Token.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-status-types>
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

    /// The Status Type value 0x03 and Status Type values in the range 0x0B until 0x0F are permanently reserved as
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
            0x0B..=0x0F => StatusType::ApplicationSpecific(value),
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

    use regex::Regex;
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    static STATUS_LIST_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"status\[(\d+)\]\s*=\s*(\d+)").unwrap());

    // Parse the status list examples as they are listed in the spec
    fn parse_status_list(input: &str) -> StatusList {
        let sparse = STATUS_LIST_REGEX
            .captures_iter(input)
            .fold(SparseStatusVec::default(), |mut acc, cap| {
                let index = cap.get(1).unwrap().as_str().parse::<usize>().unwrap();
                let value = cap.get(2).unwrap().as_str().parse::<u8>().unwrap();

                if value != 0 {
                    acc.insert(index, StatusType::from(value));
                }
                acc
            });

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
    #[case(EXAMPLE_STATUS_LIST_ONE.to_owned(), Bits::One)]
    #[case(EXAMPLE_STATUS_LIST_TWO.to_owned(), Bits::Two)]
    #[case(ONE_BIT_STATUS_LIST.to_owned(), Bits::One)]
    #[case(TWO_BIT_STATUS_LIST.to_owned(), Bits::Two)]
    #[case(FOUR_BIT_STATUS_LIST.to_owned(), Bits::Four)]
    #[case(EIGHT_BIT_STATUS_LIST.to_owned(), Bits::Eight)]
    fn test_status_list_serialization(#[case] list: StatusList, #[case] expected: Bits) {
        let packed = list.pack();
        let compressed = serde_json::to_value(packed).unwrap();
        assert_eq!(compressed["bits"].as_u64().unwrap(), expected as u64);
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
        let packed: PackedStatusList = serde_json::from_value(value).unwrap();

        let status_list = packed.unpack();

        assert_eq!(status_list.sparse, expected.sparse);
        // Since the example input only specifies non-valid entries, there can be more entries in the deserialized list
        assert!(status_list.len >= expected.len);
    }

    #[rstest]
    #[case(vec![0, 1993, 35460], vec![StatusType::Invalid, StatusType::Suspended, StatusType::ApplicationSpecific(3)])]
    #[case(vec![1, 2, 3], vec![StatusType::Valid, StatusType::Valid, StatusType::Valid])]
    #[case(vec![0, 1], vec![StatusType::Invalid, StatusType::Valid])]
    #[case(vec![1, 0], vec![StatusType::Valid, StatusType::Invalid])]
    fn test_partial_unpack(#[case] indices: Vec<usize>, #[case] expected: Vec<StatusType>) {
        let packed = FOUR_BIT_STATUS_LIST.to_owned().pack();
        let unpacked = packed.partial_unpack(&indices);
        assert_eq!(unpacked, expected);
    }
}
