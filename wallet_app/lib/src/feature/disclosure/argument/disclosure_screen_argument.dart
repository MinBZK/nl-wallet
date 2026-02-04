import 'package:freezed_annotation/freezed_annotation.dart';

part 'disclosure_screen_argument.freezed.dart';
part 'disclosure_screen_argument.g.dart';

@freezed
abstract class DisclosureScreenArgument with _$DisclosureScreenArgument {
  const factory DisclosureScreenArgument({
    required String uri,
    required bool isQrCode,
  }) = _DisclosureScreenArgument;

  factory DisclosureScreenArgument.fromJson(Map<String, dynamic> json) => _$DisclosureScreenArgumentFromJson(json);
}
