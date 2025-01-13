// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'instruction.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$WalletInstructionError {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int attemptsLeftInRound, bool isFinalRound) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int attemptsLeftInRound, bool isFinalRound)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int attemptsLeftInRound, bool isFinalRound)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletInstructionError_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionError_Timeout value) timeout,
    required TResult Function(WalletInstructionError_Blocked value) blocked,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionError_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionError_Timeout value)? timeout,
    TResult? Function(WalletInstructionError_Blocked value)? blocked,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionError_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionError_Timeout value)? timeout,
    TResult Function(WalletInstructionError_Blocked value)? blocked,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $WalletInstructionErrorCopyWith<$Res> {
  factory $WalletInstructionErrorCopyWith(WalletInstructionError value, $Res Function(WalletInstructionError) then) =
      _$WalletInstructionErrorCopyWithImpl<$Res, WalletInstructionError>;
}

/// @nodoc
class _$WalletInstructionErrorCopyWithImpl<$Res, $Val extends WalletInstructionError>
    implements $WalletInstructionErrorCopyWith<$Res> {
  _$WalletInstructionErrorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$WalletInstructionError_IncorrectPinImplCopyWith<$Res> {
  factory _$$WalletInstructionError_IncorrectPinImplCopyWith(_$WalletInstructionError_IncorrectPinImpl value,
          $Res Function(_$WalletInstructionError_IncorrectPinImpl) then) =
      __$$WalletInstructionError_IncorrectPinImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int attemptsLeftInRound, bool isFinalRound});
}

/// @nodoc
class __$$WalletInstructionError_IncorrectPinImplCopyWithImpl<$Res>
    extends _$WalletInstructionErrorCopyWithImpl<$Res, _$WalletInstructionError_IncorrectPinImpl>
    implements _$$WalletInstructionError_IncorrectPinImplCopyWith<$Res> {
  __$$WalletInstructionError_IncorrectPinImplCopyWithImpl(
      _$WalletInstructionError_IncorrectPinImpl _value, $Res Function(_$WalletInstructionError_IncorrectPinImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? attemptsLeftInRound = null,
    Object? isFinalRound = null,
  }) {
    return _then(_$WalletInstructionError_IncorrectPinImpl(
      attemptsLeftInRound: null == attemptsLeftInRound
          ? _value.attemptsLeftInRound
          : attemptsLeftInRound // ignore: cast_nullable_to_non_nullable
              as int,
      isFinalRound: null == isFinalRound
          ? _value.isFinalRound
          : isFinalRound // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc

class _$WalletInstructionError_IncorrectPinImpl extends WalletInstructionError_IncorrectPin {
  const _$WalletInstructionError_IncorrectPinImpl({required this.attemptsLeftInRound, required this.isFinalRound})
      : super._();

  @override
  final int attemptsLeftInRound;
  @override
  final bool isFinalRound;

  @override
  String toString() {
    return 'WalletInstructionError.incorrectPin(attemptsLeftInRound: $attemptsLeftInRound, isFinalRound: $isFinalRound)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletInstructionError_IncorrectPinImpl &&
            (identical(other.attemptsLeftInRound, attemptsLeftInRound) ||
                other.attemptsLeftInRound == attemptsLeftInRound) &&
            (identical(other.isFinalRound, isFinalRound) || other.isFinalRound == isFinalRound));
  }

  @override
  int get hashCode => Object.hash(runtimeType, attemptsLeftInRound, isFinalRound);

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletInstructionError_IncorrectPinImplCopyWith<_$WalletInstructionError_IncorrectPinImpl> get copyWith =>
      __$$WalletInstructionError_IncorrectPinImplCopyWithImpl<_$WalletInstructionError_IncorrectPinImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int attemptsLeftInRound, bool isFinalRound) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
  }) {
    return incorrectPin(attemptsLeftInRound, isFinalRound);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int attemptsLeftInRound, bool isFinalRound)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
  }) {
    return incorrectPin?.call(attemptsLeftInRound, isFinalRound);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int attemptsLeftInRound, bool isFinalRound)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    required TResult orElse(),
  }) {
    if (incorrectPin != null) {
      return incorrectPin(attemptsLeftInRound, isFinalRound);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletInstructionError_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionError_Timeout value) timeout,
    required TResult Function(WalletInstructionError_Blocked value) blocked,
  }) {
    return incorrectPin(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionError_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionError_Timeout value)? timeout,
    TResult? Function(WalletInstructionError_Blocked value)? blocked,
  }) {
    return incorrectPin?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionError_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionError_Timeout value)? timeout,
    TResult Function(WalletInstructionError_Blocked value)? blocked,
    required TResult orElse(),
  }) {
    if (incorrectPin != null) {
      return incorrectPin(this);
    }
    return orElse();
  }
}

abstract class WalletInstructionError_IncorrectPin extends WalletInstructionError {
  const factory WalletInstructionError_IncorrectPin(
      {required final int attemptsLeftInRound,
      required final bool isFinalRound}) = _$WalletInstructionError_IncorrectPinImpl;
  const WalletInstructionError_IncorrectPin._() : super._();

  int get attemptsLeftInRound;
  bool get isFinalRound;

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WalletInstructionError_IncorrectPinImplCopyWith<_$WalletInstructionError_IncorrectPinImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletInstructionError_TimeoutImplCopyWith<$Res> {
  factory _$$WalletInstructionError_TimeoutImplCopyWith(
          _$WalletInstructionError_TimeoutImpl value, $Res Function(_$WalletInstructionError_TimeoutImpl) then) =
      __$$WalletInstructionError_TimeoutImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int timeoutMillis});
}

/// @nodoc
class __$$WalletInstructionError_TimeoutImplCopyWithImpl<$Res>
    extends _$WalletInstructionErrorCopyWithImpl<$Res, _$WalletInstructionError_TimeoutImpl>
    implements _$$WalletInstructionError_TimeoutImplCopyWith<$Res> {
  __$$WalletInstructionError_TimeoutImplCopyWithImpl(
      _$WalletInstructionError_TimeoutImpl _value, $Res Function(_$WalletInstructionError_TimeoutImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timeoutMillis = null,
  }) {
    return _then(_$WalletInstructionError_TimeoutImpl(
      timeoutMillis: null == timeoutMillis
          ? _value.timeoutMillis
          : timeoutMillis // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$WalletInstructionError_TimeoutImpl extends WalletInstructionError_Timeout {
  const _$WalletInstructionError_TimeoutImpl({required this.timeoutMillis}) : super._();

  @override
  final int timeoutMillis;

  @override
  String toString() {
    return 'WalletInstructionError.timeout(timeoutMillis: $timeoutMillis)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletInstructionError_TimeoutImpl &&
            (identical(other.timeoutMillis, timeoutMillis) || other.timeoutMillis == timeoutMillis));
  }

  @override
  int get hashCode => Object.hash(runtimeType, timeoutMillis);

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletInstructionError_TimeoutImplCopyWith<_$WalletInstructionError_TimeoutImpl> get copyWith =>
      __$$WalletInstructionError_TimeoutImplCopyWithImpl<_$WalletInstructionError_TimeoutImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int attemptsLeftInRound, bool isFinalRound) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
  }) {
    return timeout(timeoutMillis);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int attemptsLeftInRound, bool isFinalRound)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
  }) {
    return timeout?.call(timeoutMillis);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int attemptsLeftInRound, bool isFinalRound)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    required TResult orElse(),
  }) {
    if (timeout != null) {
      return timeout(timeoutMillis);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletInstructionError_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionError_Timeout value) timeout,
    required TResult Function(WalletInstructionError_Blocked value) blocked,
  }) {
    return timeout(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionError_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionError_Timeout value)? timeout,
    TResult? Function(WalletInstructionError_Blocked value)? blocked,
  }) {
    return timeout?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionError_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionError_Timeout value)? timeout,
    TResult Function(WalletInstructionError_Blocked value)? blocked,
    required TResult orElse(),
  }) {
    if (timeout != null) {
      return timeout(this);
    }
    return orElse();
  }
}

abstract class WalletInstructionError_Timeout extends WalletInstructionError {
  const factory WalletInstructionError_Timeout({required final int timeoutMillis}) =
      _$WalletInstructionError_TimeoutImpl;
  const WalletInstructionError_Timeout._() : super._();

  int get timeoutMillis;

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WalletInstructionError_TimeoutImplCopyWith<_$WalletInstructionError_TimeoutImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletInstructionError_BlockedImplCopyWith<$Res> {
  factory _$$WalletInstructionError_BlockedImplCopyWith(
          _$WalletInstructionError_BlockedImpl value, $Res Function(_$WalletInstructionError_BlockedImpl) then) =
      __$$WalletInstructionError_BlockedImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$WalletInstructionError_BlockedImplCopyWithImpl<$Res>
    extends _$WalletInstructionErrorCopyWithImpl<$Res, _$WalletInstructionError_BlockedImpl>
    implements _$$WalletInstructionError_BlockedImplCopyWith<$Res> {
  __$$WalletInstructionError_BlockedImplCopyWithImpl(
      _$WalletInstructionError_BlockedImpl _value, $Res Function(_$WalletInstructionError_BlockedImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$WalletInstructionError_BlockedImpl extends WalletInstructionError_Blocked {
  const _$WalletInstructionError_BlockedImpl() : super._();

  @override
  String toString() {
    return 'WalletInstructionError.blocked()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is _$WalletInstructionError_BlockedImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int attemptsLeftInRound, bool isFinalRound) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
  }) {
    return blocked();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int attemptsLeftInRound, bool isFinalRound)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
  }) {
    return blocked?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int attemptsLeftInRound, bool isFinalRound)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    required TResult orElse(),
  }) {
    if (blocked != null) {
      return blocked();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletInstructionError_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionError_Timeout value) timeout,
    required TResult Function(WalletInstructionError_Blocked value) blocked,
  }) {
    return blocked(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionError_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionError_Timeout value)? timeout,
    TResult? Function(WalletInstructionError_Blocked value)? blocked,
  }) {
    return blocked?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionError_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionError_Timeout value)? timeout,
    TResult Function(WalletInstructionError_Blocked value)? blocked,
    required TResult orElse(),
  }) {
    if (blocked != null) {
      return blocked(this);
    }
    return orElse();
  }
}

abstract class WalletInstructionError_Blocked extends WalletInstructionError {
  const factory WalletInstructionError_Blocked() = _$WalletInstructionError_BlockedImpl;
  const WalletInstructionError_Blocked._() : super._();
}

/// @nodoc
mixin _$WalletInstructionResult {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(WalletInstructionError error)? instructionError,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletInstructionResult_Ok value) ok,
    required TResult Function(WalletInstructionResult_InstructionError value) instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionResult_Ok value)? ok,
    TResult? Function(WalletInstructionResult_InstructionError value)? instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionResult_Ok value)? ok,
    TResult Function(WalletInstructionResult_InstructionError value)? instructionError,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $WalletInstructionResultCopyWith<$Res> {
  factory $WalletInstructionResultCopyWith(WalletInstructionResult value, $Res Function(WalletInstructionResult) then) =
      _$WalletInstructionResultCopyWithImpl<$Res, WalletInstructionResult>;
}

/// @nodoc
class _$WalletInstructionResultCopyWithImpl<$Res, $Val extends WalletInstructionResult>
    implements $WalletInstructionResultCopyWith<$Res> {
  _$WalletInstructionResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$WalletInstructionResult_OkImplCopyWith<$Res> {
  factory _$$WalletInstructionResult_OkImplCopyWith(
          _$WalletInstructionResult_OkImpl value, $Res Function(_$WalletInstructionResult_OkImpl) then) =
      __$$WalletInstructionResult_OkImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$WalletInstructionResult_OkImplCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_OkImpl>
    implements _$$WalletInstructionResult_OkImplCopyWith<$Res> {
  __$$WalletInstructionResult_OkImplCopyWithImpl(
      _$WalletInstructionResult_OkImpl _value, $Res Function(_$WalletInstructionResult_OkImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$WalletInstructionResult_OkImpl extends WalletInstructionResult_Ok {
  const _$WalletInstructionResult_OkImpl() : super._();

  @override
  String toString() {
    return 'WalletInstructionResult.ok()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$WalletInstructionResult_OkImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) {
    return ok();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) {
    return ok?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(WalletInstructionError error)? instructionError,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletInstructionResult_Ok value) ok,
    required TResult Function(WalletInstructionResult_InstructionError value) instructionError,
  }) {
    return ok(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionResult_Ok value)? ok,
    TResult? Function(WalletInstructionResult_InstructionError value)? instructionError,
  }) {
    return ok?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionResult_Ok value)? ok,
    TResult Function(WalletInstructionResult_InstructionError value)? instructionError,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok(this);
    }
    return orElse();
  }
}

abstract class WalletInstructionResult_Ok extends WalletInstructionResult {
  const factory WalletInstructionResult_Ok() = _$WalletInstructionResult_OkImpl;
  const WalletInstructionResult_Ok._() : super._();
}

/// @nodoc
abstract class _$$WalletInstructionResult_InstructionErrorImplCopyWith<$Res> {
  factory _$$WalletInstructionResult_InstructionErrorImplCopyWith(_$WalletInstructionResult_InstructionErrorImpl value,
          $Res Function(_$WalletInstructionResult_InstructionErrorImpl) then) =
      __$$WalletInstructionResult_InstructionErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({WalletInstructionError error});

  $WalletInstructionErrorCopyWith<$Res> get error;
}

/// @nodoc
class __$$WalletInstructionResult_InstructionErrorImplCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_InstructionErrorImpl>
    implements _$$WalletInstructionResult_InstructionErrorImplCopyWith<$Res> {
  __$$WalletInstructionResult_InstructionErrorImplCopyWithImpl(_$WalletInstructionResult_InstructionErrorImpl _value,
      $Res Function(_$WalletInstructionResult_InstructionErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? error = null,
  }) {
    return _then(_$WalletInstructionResult_InstructionErrorImpl(
      error: null == error
          ? _value.error
          : error // ignore: cast_nullable_to_non_nullable
              as WalletInstructionError,
    ));
  }

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $WalletInstructionErrorCopyWith<$Res> get error {
    return $WalletInstructionErrorCopyWith<$Res>(_value.error, (value) {
      return _then(_value.copyWith(error: value));
    });
  }
}

/// @nodoc

class _$WalletInstructionResult_InstructionErrorImpl extends WalletInstructionResult_InstructionError {
  const _$WalletInstructionResult_InstructionErrorImpl({required this.error}) : super._();

  @override
  final WalletInstructionError error;

  @override
  String toString() {
    return 'WalletInstructionResult.instructionError(error: $error)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletInstructionResult_InstructionErrorImpl &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletInstructionResult_InstructionErrorImplCopyWith<_$WalletInstructionResult_InstructionErrorImpl>
      get copyWith =>
          __$$WalletInstructionResult_InstructionErrorImplCopyWithImpl<_$WalletInstructionResult_InstructionErrorImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) {
    return instructionError(error);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) {
    return instructionError?.call(error);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(WalletInstructionError error)? instructionError,
    required TResult orElse(),
  }) {
    if (instructionError != null) {
      return instructionError(error);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletInstructionResult_Ok value) ok,
    required TResult Function(WalletInstructionResult_InstructionError value) instructionError,
  }) {
    return instructionError(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionResult_Ok value)? ok,
    TResult? Function(WalletInstructionResult_InstructionError value)? instructionError,
  }) {
    return instructionError?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionResult_Ok value)? ok,
    TResult Function(WalletInstructionResult_InstructionError value)? instructionError,
    required TResult orElse(),
  }) {
    if (instructionError != null) {
      return instructionError(this);
    }
    return orElse();
  }
}

abstract class WalletInstructionResult_InstructionError extends WalletInstructionResult {
  const factory WalletInstructionResult_InstructionError({required final WalletInstructionError error}) =
      _$WalletInstructionResult_InstructionErrorImpl;
  const WalletInstructionResult_InstructionError._() : super._();

  WalletInstructionError get error;

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WalletInstructionResult_InstructionErrorImplCopyWith<_$WalletInstructionResult_InstructionErrorImpl>
      get copyWith => throw _privateConstructorUsedError;
}
