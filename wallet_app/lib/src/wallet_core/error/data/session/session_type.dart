import 'package:freezed_annotation/freezed_annotation.dart';

enum SessionType {
  @JsonValue('same_device')
  sameDevice,
  @JsonValue('cross_device')
  crossDevice,
  @JsonValue('unknown')
  unknown,
}
