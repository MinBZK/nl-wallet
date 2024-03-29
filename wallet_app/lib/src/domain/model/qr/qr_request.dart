sealed class QrRequest {
  String get sessionId;

  QrRequestType get type;
}

class QrIssuanceRequest implements QrRequest {
  @override
  final String sessionId;

  QrIssuanceRequest(this.sessionId);

  @override
  QrRequestType get type => QrRequestType.issuance;
}

class QrDisclosureRequest implements QrRequest {
  @override
  final String sessionId;

  QrDisclosureRequest(this.sessionId);

  @override
  QrRequestType get type => QrRequestType.disclosure;
}

class QrSignRequest implements QrRequest {
  @override
  final String sessionId;

  QrSignRequest(this.sessionId);

  @override
  QrRequestType get type => QrRequestType.sign;
}

enum QrRequestType { disclosure, issuance, sign }
