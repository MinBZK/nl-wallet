import 'package:freezed_annotation/freezed_annotation.dart';

part 'start_disclosure_request.freezed.dart';
part 'start_disclosure_request.g.dart';

@freezed
sealed class StartDisclosureRequest with _$StartDisclosureRequest {
  const factory StartDisclosureRequest.deeplink(String uri) = DeeplinkStartDisclosureRequest;

  const factory StartDisclosureRequest.qrScan(String uri) = QrScanStartDisclosureRequest;

  const factory StartDisclosureRequest.closeProximity() = CloseProximityStartDisclosureRequest;

  factory StartDisclosureRequest.fromJson(Map<String, dynamic> json) => _$StartDisclosureRequestFromJson(json);
}
