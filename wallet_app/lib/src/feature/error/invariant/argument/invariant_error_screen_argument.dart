import 'package:freezed_annotation/freezed_annotation.dart';

part 'invariant_error_screen_argument.freezed.dart';

part 'invariant_error_screen_argument.g.dart';

@Freezed(copyWith: false)
abstract class InvariantErrorScreenArgument with _$InvariantErrorScreenArgument {
  /// Technical error code/details, shown to (and copyable by) developers.
  const factory InvariantErrorScreenArgument({String? code}) = _InvariantErrorScreenArgument;

  const InvariantErrorScreenArgument._();

  factory InvariantErrorScreenArgument.fromJson(Map<String, dynamic> json) =>
      _$InvariantErrorScreenArgumentFromJson(json);
}
