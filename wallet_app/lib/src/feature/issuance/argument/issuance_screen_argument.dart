import 'package:freezed_annotation/freezed_annotation.dart';

part 'issuance_screen_argument.freezed.dart';
part 'issuance_screen_argument.g.dart';

@freezed
abstract class IssuanceScreenArgument with _$IssuanceScreenArgument {
  @Assert('mockSessionId != null || uri != null', 'Either a mockSessionId or a uri is needed to start issuance')
  const factory IssuanceScreenArgument({
    String? mockSessionId,
    required bool isQrCode,
    @Default(false) bool isRefreshFlow,
    String? uri,
  }) = _IssuanceScreenArgument;

  factory IssuanceScreenArgument.fromJson(Map<String, dynamic> json) => _$IssuanceScreenArgumentFromJson(json);
}
