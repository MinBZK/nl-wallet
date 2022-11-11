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

enum QrRequestType { verification, issuance }
