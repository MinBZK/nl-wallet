import 'package:freezed_annotation/freezed_annotation.dart';

part 'forgot_pin_screen_argument.freezed.dart';
part 'forgot_pin_screen_argument.g.dart';

@freezed
abstract class ForgotPinScreenArgument with _$ForgotPinScreenArgument {
  const factory ForgotPinScreenArgument({
    required bool useCloseButton,
  }) = _ForgotPinScreenArgument;

  factory ForgotPinScreenArgument.fromJson(Map<String, dynamic> json) => _$ForgotPinScreenArgumentFromJson(json);
}
