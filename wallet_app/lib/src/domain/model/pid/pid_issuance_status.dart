import '../attribute/attribute.dart';

sealed class PidIssuanceStatus {}

class PidIssuanceIdle extends PidIssuanceStatus {}

class PidIssuanceAuthenticating extends PidIssuanceStatus {}

class PidIssuanceSuccess extends PidIssuanceStatus {
  final List<Attribute> previews;

  PidIssuanceSuccess(this.previews);
}

class PidIssuanceError extends PidIssuanceStatus {}
