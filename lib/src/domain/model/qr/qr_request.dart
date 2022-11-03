abstract class QrRequest {
  String get sessionId;

  QrRequestType get type;
}

class IssuanceRequest implements QrRequest {
  @override
  final String sessionId;

  IssuanceRequest(this.sessionId);

  @override
  QrRequestType get type => QrRequestType.issuance;
}

class VerificationRequest implements QrRequest {
  @override
  final String sessionId;

  VerificationRequest(this.sessionId);

  @override
  QrRequestType get type => QrRequestType.verification;
}

enum QrRequestType { verification, issuance }
