import 'package:freezed_annotation/freezed_annotation.dart';

enum RedirectError {
  @JsonValue('access_denied')
  accessDenied,
  @JsonValue('server_error')
  serverError,
  @JsonValue('login_required')
  loginRequired,
  @JsonValue('unknown')
  unknown,
}
