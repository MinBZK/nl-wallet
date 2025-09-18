import 'package:flutter/foundation.dart';

@immutable
class DisclosureScreenArgument {
  static const _kUriKey = 'uri';
  static const _kIsQrCodeKey = 'isQrCode';

  final String uri;
  final bool isQrCode;

  const DisclosureScreenArgument({required this.uri, required this.isQrCode});

  Map<String, dynamic> toMap() {
    return {
      _kIsQrCodeKey: isQrCode,
      _kUriKey: uri,
    };
  }

  DisclosureScreenArgument.fromMap(Map<String, dynamic> map) : isQrCode = map[_kIsQrCodeKey], uri = map[_kUriKey];

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is DisclosureScreenArgument &&
          runtimeType == other.runtimeType &&
          isQrCode == other.isQrCode &&
          uri == other.uri;

  @override
  int get hashCode => Object.hash(
    runtimeType,
    isQrCode,
    uri,
  );

  @override
  String toString() {
    return 'DisclosureScreenArgument{uri: $uri, isQrCode: $isQrCode}';
  }
}
