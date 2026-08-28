import 'package:freezed_annotation/freezed_annotation.dart';

part 'sign_screen_argument.freezed.dart';
part 'sign_screen_argument.g.dart';

@freezed
abstract class SignScreenArgument with _$SignScreenArgument {
  const factory SignScreenArgument({
    required String uri,
  }) = _SignScreenArgument;

  factory SignScreenArgument.fromJson(Map<String, dynamic> json) => _$SignScreenArgumentFromJson(json);
}
