/// ISO 3166-1 alpha-2 country codes, covering every code currently assigned by the standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum CountryCode {
    #[strum(serialize = "AD")]
    Andorra,
    #[strum(serialize = "AE")]
    UnitedArabEmirates,
    #[strum(serialize = "AF")]
    Afghanistan,
    #[strum(serialize = "AG")]
    AntiguaAndBarbuda,
    #[strum(serialize = "AI")]
    Anguilla,
    #[strum(serialize = "AL")]
    Albania,
    #[strum(serialize = "AM")]
    Armenia,
    #[strum(serialize = "AO")]
    Angola,
    #[strum(serialize = "AQ")]
    Antarctica,
    #[strum(serialize = "AR")]
    Argentina,
    #[strum(serialize = "AS")]
    AmericanSamoa,
    #[strum(serialize = "AT")]
    Austria,
    #[strum(serialize = "AU")]
    Australia,
    #[strum(serialize = "AW")]
    Aruba,
    #[strum(serialize = "AX")]
    AlandIslands,
    #[strum(serialize = "AZ")]
    Azerbaijan,
    #[strum(serialize = "BA")]
    BosniaAndHerzegovina,
    #[strum(serialize = "BB")]
    Barbados,
    #[strum(serialize = "BD")]
    Bangladesh,
    #[strum(serialize = "BE")]
    Belgium,
    #[strum(serialize = "BF")]
    BurkinaFaso,
    #[strum(serialize = "BG")]
    Bulgaria,
    #[strum(serialize = "BH")]
    Bahrain,
    #[strum(serialize = "BI")]
    Burundi,
    #[strum(serialize = "BJ")]
    Benin,
    #[strum(serialize = "BL")]
    SaintBarthelemy,
    #[strum(serialize = "BM")]
    Bermuda,
    #[strum(serialize = "BN")]
    BruneiDarussalam,
    #[strum(serialize = "BO")]
    Bolivia,
    #[strum(serialize = "BQ")]
    BonaireSintEustatiusAndSaba,
    #[strum(serialize = "BR")]
    Brazil,
    #[strum(serialize = "BS")]
    Bahamas,
    #[strum(serialize = "BT")]
    Bhutan,
    #[strum(serialize = "BV")]
    BouvetIsland,
    #[strum(serialize = "BW")]
    Botswana,
    #[strum(serialize = "BY")]
    Belarus,
    #[strum(serialize = "BZ")]
    Belize,
    #[strum(serialize = "CA")]
    Canada,
    #[strum(serialize = "CC")]
    CocosKeelingIslands,
    #[strum(serialize = "CD")]
    DemocraticRepublicOfTheCongo,
    #[strum(serialize = "CF")]
    CentralAfricanRepublic,
    #[strum(serialize = "CG")]
    Congo,
    #[strum(serialize = "CH")]
    Switzerland,
    #[strum(serialize = "CI")]
    CoteDIvoire,
    #[strum(serialize = "CK")]
    CookIslands,
    #[strum(serialize = "CL")]
    Chile,
    #[strum(serialize = "CM")]
    Cameroon,
    #[strum(serialize = "CN")]
    China,
    #[strum(serialize = "CO")]
    Colombia,
    #[strum(serialize = "CR")]
    CostaRica,
    #[strum(serialize = "CU")]
    Cuba,
    #[strum(serialize = "CV")]
    CaboVerde,
    #[strum(serialize = "CW")]
    Curacao,
    #[strum(serialize = "CX")]
    ChristmasIsland,
    #[strum(serialize = "CY")]
    Cyprus,
    #[strum(serialize = "CZ")]
    Czechia,
    #[strum(serialize = "DE")]
    Germany,
    #[strum(serialize = "DJ")]
    Djibouti,
    #[strum(serialize = "DK")]
    Denmark,
    #[strum(serialize = "DM")]
    Dominica,
    #[strum(serialize = "DO")]
    DominicanRepublic,
    #[strum(serialize = "DZ")]
    Algeria,
    #[strum(serialize = "EC")]
    Ecuador,
    #[strum(serialize = "EE")]
    Estonia,
    #[strum(serialize = "EG")]
    Egypt,
    #[strum(serialize = "EH")]
    WesternSahara,
    #[strum(serialize = "ER")]
    Eritrea,
    #[strum(serialize = "ES")]
    Spain,
    #[strum(serialize = "ET")]
    Ethiopia,
    #[strum(serialize = "FI")]
    Finland,
    #[strum(serialize = "FJ")]
    Fiji,
    #[strum(serialize = "FK")]
    FalklandIslands,
    #[strum(serialize = "FM")]
    Micronesia,
    #[strum(serialize = "FO")]
    FaroeIslands,
    #[strum(serialize = "FR")]
    France,
    #[strum(serialize = "GA")]
    Gabon,
    #[strum(serialize = "GB")]
    UnitedKingdom,
    #[strum(serialize = "GD")]
    Grenada,
    #[strum(serialize = "GE")]
    Georgia,
    #[strum(serialize = "GF")]
    FrenchGuiana,
    #[strum(serialize = "GG")]
    Guernsey,
    #[strum(serialize = "GH")]
    Ghana,
    #[strum(serialize = "GI")]
    Gibraltar,
    #[strum(serialize = "GL")]
    Greenland,
    #[strum(serialize = "GM")]
    Gambia,
    #[strum(serialize = "GN")]
    Guinea,
    #[strum(serialize = "GP")]
    Guadeloupe,
    #[strum(serialize = "GQ")]
    EquatorialGuinea,
    #[strum(serialize = "GR")]
    Greece,
    #[strum(serialize = "GS")]
    SouthGeorgiaAndTheSouthSandwichIslands,
    #[strum(serialize = "GT")]
    Guatemala,
    #[strum(serialize = "GU")]
    Guam,
    #[strum(serialize = "GW")]
    GuineaBissau,
    #[strum(serialize = "GY")]
    Guyana,
    #[strum(serialize = "HK")]
    HongKong,
    #[strum(serialize = "HM")]
    HeardIslandAndMcDonaldIslands,
    #[strum(serialize = "HN")]
    Honduras,
    #[strum(serialize = "HR")]
    Croatia,
    #[strum(serialize = "HT")]
    Haiti,
    #[strum(serialize = "HU")]
    Hungary,
    #[strum(serialize = "ID")]
    Indonesia,
    #[strum(serialize = "IE")]
    Ireland,
    #[strum(serialize = "IL")]
    Israel,
    #[strum(serialize = "IM")]
    IsleOfMan,
    #[strum(serialize = "IN")]
    India,
    #[strum(serialize = "IO")]
    BritishIndianOceanTerritory,
    #[strum(serialize = "IQ")]
    Iraq,
    #[strum(serialize = "IR")]
    Iran,
    #[strum(serialize = "IS")]
    Iceland,
    #[strum(serialize = "IT")]
    Italy,
    #[strum(serialize = "JE")]
    Jersey,
    #[strum(serialize = "JM")]
    Jamaica,
    #[strum(serialize = "JO")]
    Jordan,
    #[strum(serialize = "JP")]
    Japan,
    #[strum(serialize = "KE")]
    Kenya,
    #[strum(serialize = "KG")]
    Kyrgyzstan,
    #[strum(serialize = "KH")]
    Cambodia,
    #[strum(serialize = "KI")]
    Kiribati,
    #[strum(serialize = "KM")]
    Comoros,
    #[strum(serialize = "KN")]
    SaintKittsAndNevis,
    #[strum(serialize = "KP")]
    NorthKorea,
    #[strum(serialize = "KR")]
    SouthKorea,
    #[strum(serialize = "KW")]
    Kuwait,
    #[strum(serialize = "KY")]
    CaymanIslands,
    #[strum(serialize = "KZ")]
    Kazakhstan,
    #[strum(serialize = "LA")]
    Laos,
    #[strum(serialize = "LB")]
    Lebanon,
    #[strum(serialize = "LC")]
    SaintLucia,
    #[strum(serialize = "LI")]
    Liechtenstein,
    #[strum(serialize = "LK")]
    SriLanka,
    #[strum(serialize = "LR")]
    Liberia,
    #[strum(serialize = "LS")]
    Lesotho,
    #[strum(serialize = "LT")]
    Lithuania,
    #[strum(serialize = "LU")]
    Luxembourg,
    #[strum(serialize = "LV")]
    Latvia,
    #[strum(serialize = "LY")]
    Libya,
    #[strum(serialize = "MA")]
    Morocco,
    #[strum(serialize = "MC")]
    Monaco,
    #[strum(serialize = "MD")]
    Moldova,
    #[strum(serialize = "ME")]
    Montenegro,
    #[strum(serialize = "MF")]
    SaintMartin,
    #[strum(serialize = "MG")]
    Madagascar,
    #[strum(serialize = "MH")]
    MarshallIslands,
    #[strum(serialize = "MK")]
    NorthMacedonia,
    #[strum(serialize = "ML")]
    Mali,
    #[strum(serialize = "MM")]
    Myanmar,
    #[strum(serialize = "MN")]
    Mongolia,
    #[strum(serialize = "MO")]
    Macao,
    #[strum(serialize = "MP")]
    NorthernMarianaIslands,
    #[strum(serialize = "MQ")]
    Martinique,
    #[strum(serialize = "MR")]
    Mauritania,
    #[strum(serialize = "MS")]
    Montserrat,
    #[strum(serialize = "MT")]
    Malta,
    #[strum(serialize = "MU")]
    Mauritius,
    #[strum(serialize = "MV")]
    Maldives,
    #[strum(serialize = "MW")]
    Malawi,
    #[strum(serialize = "MX")]
    Mexico,
    #[strum(serialize = "MY")]
    Malaysia,
    #[strum(serialize = "MZ")]
    Mozambique,
    #[strum(serialize = "NA")]
    Namibia,
    #[strum(serialize = "NC")]
    NewCaledonia,
    #[strum(serialize = "NE")]
    Niger,
    #[strum(serialize = "NF")]
    NorfolkIsland,
    #[strum(serialize = "NG")]
    Nigeria,
    #[strum(serialize = "NI")]
    Nicaragua,
    #[strum(serialize = "NL")]
    Netherlands,
    #[strum(serialize = "NO")]
    Norway,
    #[strum(serialize = "NP")]
    Nepal,
    #[strum(serialize = "NR")]
    Nauru,
    #[strum(serialize = "NU")]
    Niue,
    #[strum(serialize = "NZ")]
    NewZealand,
    #[strum(serialize = "OM")]
    Oman,
    #[strum(serialize = "PA")]
    Panama,
    #[strum(serialize = "PE")]
    Peru,
    #[strum(serialize = "PF")]
    FrenchPolynesia,
    #[strum(serialize = "PG")]
    PapuaNewGuinea,
    #[strum(serialize = "PH")]
    Philippines,
    #[strum(serialize = "PK")]
    Pakistan,
    #[strum(serialize = "PL")]
    Poland,
    #[strum(serialize = "PM")]
    SaintPierreAndMiquelon,
    #[strum(serialize = "PN")]
    Pitcairn,
    #[strum(serialize = "PR")]
    PuertoRico,
    #[strum(serialize = "PS")]
    Palestine,
    #[strum(serialize = "PT")]
    Portugal,
    #[strum(serialize = "PW")]
    Palau,
    #[strum(serialize = "PY")]
    Paraguay,
    #[strum(serialize = "QA")]
    Qatar,
    #[strum(serialize = "RE")]
    Reunion,
    #[strum(serialize = "RO")]
    Romania,
    #[strum(serialize = "RS")]
    Serbia,
    #[strum(serialize = "RU")]
    Russia,
    #[strum(serialize = "RW")]
    Rwanda,
    #[strum(serialize = "SA")]
    SaudiArabia,
    #[strum(serialize = "SB")]
    SolomonIslands,
    #[strum(serialize = "SC")]
    Seychelles,
    #[strum(serialize = "SD")]
    Sudan,
    #[strum(serialize = "SE")]
    Sweden,
    #[strum(serialize = "SG")]
    Singapore,
    #[strum(serialize = "SH")]
    SaintHelenaAscensionAndTristanDaCunha,
    #[strum(serialize = "SI")]
    Slovenia,
    #[strum(serialize = "SJ")]
    SvalbardAndJanMayen,
    #[strum(serialize = "SK")]
    Slovakia,
    #[strum(serialize = "SL")]
    SierraLeone,
    #[strum(serialize = "SM")]
    SanMarino,
    #[strum(serialize = "SN")]
    Senegal,
    #[strum(serialize = "SO")]
    Somalia,
    #[strum(serialize = "SR")]
    Suriname,
    #[strum(serialize = "SS")]
    SouthSudan,
    #[strum(serialize = "ST")]
    SaoTomeAndPrincipe,
    #[strum(serialize = "SV")]
    ElSalvador,
    #[strum(serialize = "SX")]
    SintMaarten,
    #[strum(serialize = "SY")]
    Syria,
    #[strum(serialize = "SZ")]
    Eswatini,
    #[strum(serialize = "TC")]
    TurksAndCaicosIslands,
    #[strum(serialize = "TD")]
    Chad,
    #[strum(serialize = "TF")]
    FrenchSouthernTerritories,
    #[strum(serialize = "TG")]
    Togo,
    #[strum(serialize = "TH")]
    Thailand,
    #[strum(serialize = "TJ")]
    Tajikistan,
    #[strum(serialize = "TK")]
    Tokelau,
    #[strum(serialize = "TL")]
    TimorLeste,
    #[strum(serialize = "TM")]
    Turkmenistan,
    #[strum(serialize = "TN")]
    Tunisia,
    #[strum(serialize = "TO")]
    Tonga,
    #[strum(serialize = "TR")]
    Turkiye,
    #[strum(serialize = "TT")]
    TrinidadAndTobago,
    #[strum(serialize = "TV")]
    Tuvalu,
    #[strum(serialize = "TW")]
    Taiwan,
    #[strum(serialize = "TZ")]
    Tanzania,
    #[strum(serialize = "UA")]
    Ukraine,
    #[strum(serialize = "UG")]
    Uganda,
    #[strum(serialize = "UM")]
    UnitedStatesMinorOutlyingIslands,
    #[strum(serialize = "US")]
    UnitedStates,
    #[strum(serialize = "UY")]
    Uruguay,
    #[strum(serialize = "UZ")]
    Uzbekistan,
    #[strum(serialize = "VA")]
    HolySee,
    #[strum(serialize = "VC")]
    SaintVincentAndTheGrenadines,
    #[strum(serialize = "VE")]
    Venezuela,
    #[strum(serialize = "VG")]
    BritishVirginIslands,
    #[strum(serialize = "VI")]
    UsVirginIslands,
    #[strum(serialize = "VN")]
    Vietnam,
    #[strum(serialize = "VU")]
    Vanuatu,
    #[strum(serialize = "WF")]
    WallisAndFutuna,
    #[strum(serialize = "WS")]
    Samoa,
    #[strum(serialize = "YE")]
    Yemen,
    #[strum(serialize = "YT")]
    Mayotte,
    #[strum(serialize = "ZA")]
    SouthAfrica,
    #[strum(serialize = "ZM")]
    Zambia,
    #[strum(serialize = "ZW")]
    Zimbabwe,
}
