import 'package:freezed_annotation/freezed_annotation.dart';

part 'sign_screen_argument.freezed.dart';
part 'sign_screen_argument.g.dart';

@freezed
abstract class SignScreenArgument with _$SignScreenArgument {
  @Assert('mockSessionId != null || uri != null', 'Either a mockSessionId or a uri is needed to start signing')
  const factory SignScreenArgument({
    String? mockSessionId,
    String? uri,
  }) = _SignScreenArgument;

  factory SignScreenArgument.fromJson(Map<String, dynamic> json) => _$SignScreenArgumentFromJson(json);
}
