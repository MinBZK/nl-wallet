import 'package:freezed_annotation/freezed_annotation.dart';

part 'disclosure_screen_argument.freezed.dart';
part 'disclosure_screen_argument.g.dart';

@freezed
abstract class DisclosureScreenArgument with _$DisclosureScreenArgument {
  const factory DisclosureScreenArgument({
    required DisclosureConnectionType type,
  }) = _DisclosureScreenArgument;

  factory DisclosureScreenArgument.fromJson(Map<String, dynamic> json) => _$DisclosureScreenArgumentFromJson(json);
}

@freezed
sealed class DisclosureConnectionType with _$DisclosureConnectionType {
  const factory DisclosureConnectionType.remote(
    String uri, {
    required bool isQrCode,
  }) = RemoteDisclosure;

  const factory DisclosureConnectionType.closeProximity() = CloseProximityDisclosure;

  factory DisclosureConnectionType.fromJson(Map<String, dynamic> json) => _$DisclosureConnectionTypeFromJson(json);
}
