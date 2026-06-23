/// The type of issuance flow, deduced from the initial deeplink URI.
enum IssuanceType {
  /// Start issuance via card disclosure. Calls `core.startIssuance()`.
  disclosureBasedIssuance,

  /// Start issuance via OpenID4VCI offer. Calls `core.startIssuanceFromOffer()`.
  credentialOffer,

  /// Resume issuance after external authorization. Calls `core.continueIssuance()`.
  authorizationCallback,
}
