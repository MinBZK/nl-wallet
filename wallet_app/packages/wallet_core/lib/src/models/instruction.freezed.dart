// dart format width=80
// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'instruction.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$DisclosureBasedIssuanceResult {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is DisclosureBasedIssuanceResult);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DisclosureBasedIssuanceResult()';
  }
}

/// @nodoc
class $DisclosureBasedIssuanceResultCopyWith<$Res> {
  $DisclosureBasedIssuanceResultCopyWith(
      DisclosureBasedIssuanceResult _, $Res Function(DisclosureBasedIssuanceResult) __);
}

/// @nodoc

class DisclosureBasedIssuanceResult_Ok extends DisclosureBasedIssuanceResult {
  const DisclosureBasedIssuanceResult_Ok(final List<Attestation> field0)
      : _field0 = field0,
        super._();

  final List<Attestation> _field0;
  List<Attestation> get field0 {
    if (_field0 is EqualUnmodifiableListView) return _field0;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_field0);
  }

  /// Create a copy of DisclosureBasedIssuanceResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $DisclosureBasedIssuanceResult_OkCopyWith<DisclosureBasedIssuanceResult_Ok> get copyWith =>
      _$DisclosureBasedIssuanceResult_OkCopyWithImpl<DisclosureBasedIssuanceResult_Ok>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DisclosureBasedIssuanceResult_Ok &&
            const DeepCollectionEquality().equals(other._field0, _field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, const DeepCollectionEquality().hash(_field0));

  @override
  String toString() {
    return 'DisclosureBasedIssuanceResult.ok(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $DisclosureBasedIssuanceResult_OkCopyWith<$Res>
    implements $DisclosureBasedIssuanceResultCopyWith<$Res> {
  factory $DisclosureBasedIssuanceResult_OkCopyWith(
          DisclosureBasedIssuanceResult_Ok value, $Res Function(DisclosureBasedIssuanceResult_Ok) _then) =
      _$DisclosureBasedIssuanceResult_OkCopyWithImpl;
  @useResult
  $Res call({List<Attestation> field0});
}

/// @nodoc
class _$DisclosureBasedIssuanceResult_OkCopyWithImpl<$Res> implements $DisclosureBasedIssuanceResult_OkCopyWith<$Res> {
  _$DisclosureBasedIssuanceResult_OkCopyWithImpl(this._self, this._then);

  final DisclosureBasedIssuanceResult_Ok _self;
  final $Res Function(DisclosureBasedIssuanceResult_Ok) _then;

  /// Create a copy of DisclosureBasedIssuanceResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(DisclosureBasedIssuanceResult_Ok(
      null == field0
          ? _self._field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as List<Attestation>,
    ));
  }
}

/// @nodoc

class DisclosureBasedIssuanceResult_InstructionError extends DisclosureBasedIssuanceResult {
  const DisclosureBasedIssuanceResult_InstructionError({required this.error}) : super._();

  final WalletInstructionError error;

  /// Create a copy of DisclosureBasedIssuanceResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $DisclosureBasedIssuanceResult_InstructionErrorCopyWith<DisclosureBasedIssuanceResult_InstructionError>
      get copyWith =>
          _$DisclosureBasedIssuanceResult_InstructionErrorCopyWithImpl<DisclosureBasedIssuanceResult_InstructionError>(
              this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DisclosureBasedIssuanceResult_InstructionError &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  @override
  String toString() {
    return 'DisclosureBasedIssuanceResult.instructionError(error: $error)';
  }
}

/// @nodoc
abstract mixin class $DisclosureBasedIssuanceResult_InstructionErrorCopyWith<$Res>
    implements $DisclosureBasedIssuanceResultCopyWith<$Res> {
  factory $DisclosureBasedIssuanceResult_InstructionErrorCopyWith(DisclosureBasedIssuanceResult_InstructionError value,
          $Res Function(DisclosureBasedIssuanceResult_InstructionError) _then) =
      _$DisclosureBasedIssuanceResult_InstructionErrorCopyWithImpl;
  @useResult
  $Res call({WalletInstructionError error});

  $WalletInstructionErrorCopyWith<$Res> get error;
}

/// @nodoc
class _$DisclosureBasedIssuanceResult_InstructionErrorCopyWithImpl<$Res>
    implements $DisclosureBasedIssuanceResult_InstructionErrorCopyWith<$Res> {
  _$DisclosureBasedIssuanceResult_InstructionErrorCopyWithImpl(this._self, this._then);

  final DisclosureBasedIssuanceResult_InstructionError _self;
  final $Res Function(DisclosureBasedIssuanceResult_InstructionError) _then;

  /// Create a copy of DisclosureBasedIssuanceResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? error = null,
  }) {
    return _then(DisclosureBasedIssuanceResult_InstructionError(
      error: null == error
          ? _self.error
          : error // ignore: cast_nullable_to_non_nullable
              as WalletInstructionError,
    ));
  }

  /// Create a copy of DisclosureBasedIssuanceResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $WalletInstructionErrorCopyWith<$Res> get error {
    return $WalletInstructionErrorCopyWith<$Res>(_self.error, (value) {
      return _then(_self.copyWith(error: value));
    });
  }
}

/// @nodoc
mixin _$WalletInstructionError {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is WalletInstructionError);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'WalletInstructionError()';
  }
}

/// @nodoc
class $WalletInstructionErrorCopyWith<$Res> {
  $WalletInstructionErrorCopyWith(WalletInstructionError _, $Res Function(WalletInstructionError) __);
}

/// @nodoc

class WalletInstructionError_IncorrectPin extends WalletInstructionError {
  const WalletInstructionError_IncorrectPin({required this.attemptsLeftInRound, required this.isFinalRound})
      : super._();

  final int attemptsLeftInRound;
  final bool isFinalRound;

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $WalletInstructionError_IncorrectPinCopyWith<WalletInstructionError_IncorrectPin> get copyWith =>
      _$WalletInstructionError_IncorrectPinCopyWithImpl<WalletInstructionError_IncorrectPin>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is WalletInstructionError_IncorrectPin &&
            (identical(other.attemptsLeftInRound, attemptsLeftInRound) ||
                other.attemptsLeftInRound == attemptsLeftInRound) &&
            (identical(other.isFinalRound, isFinalRound) || other.isFinalRound == isFinalRound));
  }

  @override
  int get hashCode => Object.hash(runtimeType, attemptsLeftInRound, isFinalRound);

  @override
  String toString() {
    return 'WalletInstructionError.incorrectPin(attemptsLeftInRound: $attemptsLeftInRound, isFinalRound: $isFinalRound)';
  }
}

/// @nodoc
abstract mixin class $WalletInstructionError_IncorrectPinCopyWith<$Res>
    implements $WalletInstructionErrorCopyWith<$Res> {
  factory $WalletInstructionError_IncorrectPinCopyWith(
          WalletInstructionError_IncorrectPin value, $Res Function(WalletInstructionError_IncorrectPin) _then) =
      _$WalletInstructionError_IncorrectPinCopyWithImpl;
  @useResult
  $Res call({int attemptsLeftInRound, bool isFinalRound});
}

/// @nodoc
class _$WalletInstructionError_IncorrectPinCopyWithImpl<$Res>
    implements $WalletInstructionError_IncorrectPinCopyWith<$Res> {
  _$WalletInstructionError_IncorrectPinCopyWithImpl(this._self, this._then);

  final WalletInstructionError_IncorrectPin _self;
  final $Res Function(WalletInstructionError_IncorrectPin) _then;

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? attemptsLeftInRound = null,
    Object? isFinalRound = null,
  }) {
    return _then(WalletInstructionError_IncorrectPin(
      attemptsLeftInRound: null == attemptsLeftInRound
          ? _self.attemptsLeftInRound
          : attemptsLeftInRound // ignore: cast_nullable_to_non_nullable
              as int,
      isFinalRound: null == isFinalRound
          ? _self.isFinalRound
          : isFinalRound // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc

class WalletInstructionError_Timeout extends WalletInstructionError {
  const WalletInstructionError_Timeout({required this.timeoutMillis}) : super._();

  final BigInt timeoutMillis;

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $WalletInstructionError_TimeoutCopyWith<WalletInstructionError_Timeout> get copyWith =>
      _$WalletInstructionError_TimeoutCopyWithImpl<WalletInstructionError_Timeout>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is WalletInstructionError_Timeout &&
            (identical(other.timeoutMillis, timeoutMillis) || other.timeoutMillis == timeoutMillis));
  }

  @override
  int get hashCode => Object.hash(runtimeType, timeoutMillis);

  @override
  String toString() {
    return 'WalletInstructionError.timeout(timeoutMillis: $timeoutMillis)';
  }
}

/// @nodoc
abstract mixin class $WalletInstructionError_TimeoutCopyWith<$Res> implements $WalletInstructionErrorCopyWith<$Res> {
  factory $WalletInstructionError_TimeoutCopyWith(
          WalletInstructionError_Timeout value, $Res Function(WalletInstructionError_Timeout) _then) =
      _$WalletInstructionError_TimeoutCopyWithImpl;
  @useResult
  $Res call({BigInt timeoutMillis});
}

/// @nodoc
class _$WalletInstructionError_TimeoutCopyWithImpl<$Res> implements $WalletInstructionError_TimeoutCopyWith<$Res> {
  _$WalletInstructionError_TimeoutCopyWithImpl(this._self, this._then);

  final WalletInstructionError_Timeout _self;
  final $Res Function(WalletInstructionError_Timeout) _then;

  /// Create a copy of WalletInstructionError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? timeoutMillis = null,
  }) {
    return _then(WalletInstructionError_Timeout(
      timeoutMillis: null == timeoutMillis
          ? _self.timeoutMillis
          : timeoutMillis // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc

class WalletInstructionError_Blocked extends WalletInstructionError {
  const WalletInstructionError_Blocked() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is WalletInstructionError_Blocked);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'WalletInstructionError.blocked()';
  }
}

/// @nodoc
mixin _$WalletInstructionResult {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is WalletInstructionResult);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'WalletInstructionResult()';
  }
}

/// @nodoc
class $WalletInstructionResultCopyWith<$Res> {
  $WalletInstructionResultCopyWith(WalletInstructionResult _, $Res Function(WalletInstructionResult) __);
}

/// @nodoc

class WalletInstructionResult_Ok extends WalletInstructionResult {
  const WalletInstructionResult_Ok() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is WalletInstructionResult_Ok);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'WalletInstructionResult.ok()';
  }
}

/// @nodoc

class WalletInstructionResult_InstructionError extends WalletInstructionResult {
  const WalletInstructionResult_InstructionError({required this.error}) : super._();

  final WalletInstructionError error;

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $WalletInstructionResult_InstructionErrorCopyWith<WalletInstructionResult_InstructionError> get copyWith =>
      _$WalletInstructionResult_InstructionErrorCopyWithImpl<WalletInstructionResult_InstructionError>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is WalletInstructionResult_InstructionError &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  @override
  String toString() {
    return 'WalletInstructionResult.instructionError(error: $error)';
  }
}

/// @nodoc
abstract mixin class $WalletInstructionResult_InstructionErrorCopyWith<$Res>
    implements $WalletInstructionResultCopyWith<$Res> {
  factory $WalletInstructionResult_InstructionErrorCopyWith(WalletInstructionResult_InstructionError value,
          $Res Function(WalletInstructionResult_InstructionError) _then) =
      _$WalletInstructionResult_InstructionErrorCopyWithImpl;
  @useResult
  $Res call({WalletInstructionError error});

  $WalletInstructionErrorCopyWith<$Res> get error;
}

/// @nodoc
class _$WalletInstructionResult_InstructionErrorCopyWithImpl<$Res>
    implements $WalletInstructionResult_InstructionErrorCopyWith<$Res> {
  _$WalletInstructionResult_InstructionErrorCopyWithImpl(this._self, this._then);

  final WalletInstructionResult_InstructionError _self;
  final $Res Function(WalletInstructionResult_InstructionError) _then;

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? error = null,
  }) {
    return _then(WalletInstructionResult_InstructionError(
      error: null == error
          ? _self.error
          : error // ignore: cast_nullable_to_non_nullable
              as WalletInstructionError,
    ));
  }

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $WalletInstructionErrorCopyWith<$Res> get error {
    return $WalletInstructionErrorCopyWith<$Res>(_self.error, (value) {
      return _then(_self.copyWith(error: value));
    });
  }
}

// dart format on
