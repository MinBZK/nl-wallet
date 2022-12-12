abstract class QrRequest {
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

class QrVerificationRequest implements QrRequest {
  @override
  final String sessionId;

  QrVerificationRequest(this.sessionId);

  @override
  QrRequestType get type => QrRequestType.verification;
}

class QrSignRequest implements QrRequest {
  @override
  final String sessionId;

  QrSignRequest(this.sessionId);

  @override
  QrRequestType get type => QrRequestType.sign;
}

enum QrRequestType { verification, issuance, sign }
