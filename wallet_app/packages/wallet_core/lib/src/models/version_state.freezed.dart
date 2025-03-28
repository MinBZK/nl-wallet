// dart format width=80
// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'version_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$FlutterVersionState {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is FlutterVersionState);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'FlutterVersionState()';
  }
}

/// @nodoc
class $FlutterVersionStateCopyWith<$Res> {
  $FlutterVersionStateCopyWith(FlutterVersionState _, $Res Function(FlutterVersionState) __);
}

/// @nodoc

class FlutterVersionState_Ok extends FlutterVersionState {
  const FlutterVersionState_Ok() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is FlutterVersionState_Ok);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'FlutterVersionState.ok()';
  }
}

/// @nodoc

class FlutterVersionState_Notify extends FlutterVersionState {
  const FlutterVersionState_Notify() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is FlutterVersionState_Notify);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'FlutterVersionState.notify()';
  }
}

/// @nodoc

class FlutterVersionState_Recommend extends FlutterVersionState {
  const FlutterVersionState_Recommend() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is FlutterVersionState_Recommend);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'FlutterVersionState.recommend()';
  }
}

/// @nodoc

class FlutterVersionState_Warn extends FlutterVersionState {
  const FlutterVersionState_Warn({required this.expiresInSeconds}) : super._();

  final BigInt expiresInSeconds;

  /// Create a copy of FlutterVersionState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $FlutterVersionState_WarnCopyWith<FlutterVersionState_Warn> get copyWith =>
      _$FlutterVersionState_WarnCopyWithImpl<FlutterVersionState_Warn>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is FlutterVersionState_Warn &&
            (identical(other.expiresInSeconds, expiresInSeconds) || other.expiresInSeconds == expiresInSeconds));
  }

  @override
  int get hashCode => Object.hash(runtimeType, expiresInSeconds);

  @override
  String toString() {
    return 'FlutterVersionState.warn(expiresInSeconds: $expiresInSeconds)';
  }
}

/// @nodoc
abstract mixin class $FlutterVersionState_WarnCopyWith<$Res> implements $FlutterVersionStateCopyWith<$Res> {
  factory $FlutterVersionState_WarnCopyWith(
          FlutterVersionState_Warn value, $Res Function(FlutterVersionState_Warn) _then) =
      _$FlutterVersionState_WarnCopyWithImpl;
  @useResult
  $Res call({BigInt expiresInSeconds});
}

/// @nodoc
class _$FlutterVersionState_WarnCopyWithImpl<$Res> implements $FlutterVersionState_WarnCopyWith<$Res> {
  _$FlutterVersionState_WarnCopyWithImpl(this._self, this._then);

  final FlutterVersionState_Warn _self;
  final $Res Function(FlutterVersionState_Warn) _then;

  /// Create a copy of FlutterVersionState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? expiresInSeconds = null,
  }) {
    return _then(FlutterVersionState_Warn(
      expiresInSeconds: null == expiresInSeconds
          ? _self.expiresInSeconds
          : expiresInSeconds // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc

class FlutterVersionState_Block extends FlutterVersionState {
  const FlutterVersionState_Block() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is FlutterVersionState_Block);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'FlutterVersionState.block()';
  }
}

// dart format on
