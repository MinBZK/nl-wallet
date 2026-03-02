import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../wallet_core/error/core_error.dart';

part 'app_blocked_screen_argument.freezed.dart';

part 'app_blocked_screen_argument.g.dart';

@Freezed(copyWith: false)
abstract class AppBlockedScreenArgument with _$AppBlockedScreenArgument {
  const factory AppBlockedScreenArgument({
    @Default(RevocationReason.unknown) RevocationReason reason,
  }) = _AppBlockedScreenArgument;

  const AppBlockedScreenArgument._();

  factory AppBlockedScreenArgument.fromJson(Map<String, dynamic> json) => _$AppBlockedScreenArgumentFromJson(json);
}
