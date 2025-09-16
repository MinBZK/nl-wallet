// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
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

/// Adds pattern-matching-related methods to [FlutterVersionState].
extension FlutterVersionStatePatterns on FlutterVersionState {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterVersionState_Ok value)? ok,
    TResult Function(FlutterVersionState_Notify value)? notify,
    TResult Function(FlutterVersionState_Recommend value)? recommend,
    TResult Function(FlutterVersionState_Warn value)? warn,
    TResult Function(FlutterVersionState_Block value)? block,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case FlutterVersionState_Ok() when ok != null:
        return ok(_that);
      case FlutterVersionState_Notify() when notify != null:
        return notify(_that);
      case FlutterVersionState_Recommend() when recommend != null:
        return recommend(_that);
      case FlutterVersionState_Warn() when warn != null:
        return warn(_that);
      case FlutterVersionState_Block() when block != null:
        return block(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterVersionState_Ok value) ok,
    required TResult Function(FlutterVersionState_Notify value) notify,
    required TResult Function(FlutterVersionState_Recommend value) recommend,
    required TResult Function(FlutterVersionState_Warn value) warn,
    required TResult Function(FlutterVersionState_Block value) block,
  }) {
    final _that = this;
    switch (_that) {
      case FlutterVersionState_Ok():
        return ok(_that);
      case FlutterVersionState_Notify():
        return notify(_that);
      case FlutterVersionState_Recommend():
        return recommend(_that);
      case FlutterVersionState_Warn():
        return warn(_that);
      case FlutterVersionState_Block():
        return block(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterVersionState_Ok value)? ok,
    TResult? Function(FlutterVersionState_Notify value)? notify,
    TResult? Function(FlutterVersionState_Recommend value)? recommend,
    TResult? Function(FlutterVersionState_Warn value)? warn,
    TResult? Function(FlutterVersionState_Block value)? block,
  }) {
    final _that = this;
    switch (_that) {
      case FlutterVersionState_Ok() when ok != null:
        return ok(_that);
      case FlutterVersionState_Notify() when notify != null:
        return notify(_that);
      case FlutterVersionState_Recommend() when recommend != null:
        return recommend(_that);
      case FlutterVersionState_Warn() when warn != null:
        return warn(_that);
      case FlutterVersionState_Block() when block != null:
        return block(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function()? notify,
    TResult Function()? recommend,
    TResult Function(BigInt expiresInSeconds)? warn,
    TResult Function()? block,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case FlutterVersionState_Ok() when ok != null:
        return ok();
      case FlutterVersionState_Notify() when notify != null:
        return notify();
      case FlutterVersionState_Recommend() when recommend != null:
        return recommend();
      case FlutterVersionState_Warn() when warn != null:
        return warn(_that.expiresInSeconds);
      case FlutterVersionState_Block() when block != null:
        return block();
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function() notify,
    required TResult Function() recommend,
    required TResult Function(BigInt expiresInSeconds) warn,
    required TResult Function() block,
  }) {
    final _that = this;
    switch (_that) {
      case FlutterVersionState_Ok():
        return ok();
      case FlutterVersionState_Notify():
        return notify();
      case FlutterVersionState_Recommend():
        return recommend();
      case FlutterVersionState_Warn():
        return warn(_that.expiresInSeconds);
      case FlutterVersionState_Block():
        return block();
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function()? notify,
    TResult? Function()? recommend,
    TResult? Function(BigInt expiresInSeconds)? warn,
    TResult? Function()? block,
  }) {
    final _that = this;
    switch (_that) {
      case FlutterVersionState_Ok() when ok != null:
        return ok();
      case FlutterVersionState_Notify() when notify != null:
        return notify();
      case FlutterVersionState_Recommend() when recommend != null:
        return recommend();
      case FlutterVersionState_Warn() when warn != null:
        return warn(_that.expiresInSeconds);
      case FlutterVersionState_Block() when block != null:
        return block();
      case _:
        return null;
    }
  }
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
