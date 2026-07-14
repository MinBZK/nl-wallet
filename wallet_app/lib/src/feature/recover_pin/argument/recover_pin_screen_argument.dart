import 'package:freezed_annotation/freezed_annotation.dart';

part 'recover_pin_screen_argument.freezed.dart';
part 'recover_pin_screen_argument.g.dart';

@Freezed(copyWith: false)
abstract class RecoverPinScreenArgument with _$RecoverPinScreenArgument {
  const factory RecoverPinScreenArgument({
    String? uri,
    @Default(false) bool isRecoveryFlow,
  }) = _RecoverPinScreenArgument;

  const RecoverPinScreenArgument._();

  factory RecoverPinScreenArgument.fromJson(Map<String, dynamic> json) => _$RecoverPinScreenArgumentFromJson(json);
}
