// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'bridge_generated.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#custom-getters-and-methods');

/// @nodoc
mixin _$UriFlowEvent {
  DigidState get state => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(DigidState state) digidAuth,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(DigidState state)? digidAuth,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(DigidState state)? digidAuth,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UriFlowEvent_DigidAuth value) digidAuth,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UriFlowEvent_DigidAuth value)? digidAuth,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UriFlowEvent_DigidAuth value)? digidAuth,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  @JsonKey(ignore: true)
  $UriFlowEventCopyWith<UriFlowEvent> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UriFlowEventCopyWith<$Res> {
  factory $UriFlowEventCopyWith(UriFlowEvent value, $Res Function(UriFlowEvent) then) =
      _$UriFlowEventCopyWithImpl<$Res, UriFlowEvent>;
  @useResult
  $Res call({DigidState state});
}

/// @nodoc
class _$UriFlowEventCopyWithImpl<$Res, $Val extends UriFlowEvent> implements $UriFlowEventCopyWith<$Res> {
  _$UriFlowEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? state = null,
  }) {
    return _then(_value.copyWith(
      state: null == state
          ? _value.state
          : state // ignore: cast_nullable_to_non_nullable
              as DigidState,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$UriFlowEvent_DigidAuthCopyWith<$Res> implements $UriFlowEventCopyWith<$Res> {
  factory _$$UriFlowEvent_DigidAuthCopyWith(
          _$UriFlowEvent_DigidAuth value, $Res Function(_$UriFlowEvent_DigidAuth) then) =
      __$$UriFlowEvent_DigidAuthCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({DigidState state});
}

/// @nodoc
class __$$UriFlowEvent_DigidAuthCopyWithImpl<$Res> extends _$UriFlowEventCopyWithImpl<$Res, _$UriFlowEvent_DigidAuth>
    implements _$$UriFlowEvent_DigidAuthCopyWith<$Res> {
  __$$UriFlowEvent_DigidAuthCopyWithImpl(_$UriFlowEvent_DigidAuth _value, $Res Function(_$UriFlowEvent_DigidAuth) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? state = null,
  }) {
    return _then(_$UriFlowEvent_DigidAuth(
      state: null == state
          ? _value.state
          : state // ignore: cast_nullable_to_non_nullable
              as DigidState,
    ));
  }
}

/// @nodoc

class _$UriFlowEvent_DigidAuth implements UriFlowEvent_DigidAuth {
  const _$UriFlowEvent_DigidAuth({required this.state});

  @override
  final DigidState state;

  @override
  String toString() {
    return 'UriFlowEvent.digidAuth(state: $state)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UriFlowEvent_DigidAuth &&
            (identical(other.state, state) || other.state == state));
  }

  @override
  int get hashCode => Object.hash(runtimeType, state);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$UriFlowEvent_DigidAuthCopyWith<_$UriFlowEvent_DigidAuth> get copyWith =>
      __$$UriFlowEvent_DigidAuthCopyWithImpl<_$UriFlowEvent_DigidAuth>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(DigidState state) digidAuth,
  }) {
    return digidAuth(state);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(DigidState state)? digidAuth,
  }) {
    return digidAuth?.call(state);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(DigidState state)? digidAuth,
    required TResult orElse(),
  }) {
    if (digidAuth != null) {
      return digidAuth(state);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(UriFlowEvent_DigidAuth value) digidAuth,
  }) {
    return digidAuth(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(UriFlowEvent_DigidAuth value)? digidAuth,
  }) {
    return digidAuth?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(UriFlowEvent_DigidAuth value)? digidAuth,
    required TResult orElse(),
  }) {
    if (digidAuth != null) {
      return digidAuth(this);
    }
    return orElse();
  }
}

abstract class UriFlowEvent_DigidAuth implements UriFlowEvent {
  const factory UriFlowEvent_DigidAuth({required final DigidState state}) = _$UriFlowEvent_DigidAuth;

  @override
  DigidState get state;
  @override
  @JsonKey(ignore: true)
  _$$UriFlowEvent_DigidAuthCopyWith<_$UriFlowEvent_DigidAuth> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$WalletUnlockResult {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
    required TResult Function() serverError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
    TResult? Function()? serverError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    TResult Function()? serverError,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletUnlockResult_Ok value) ok,
    required TResult Function(WalletUnlockResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletUnlockResult_Timeout value) timeout,
    required TResult Function(WalletUnlockResult_Blocked value) blocked,
    required TResult Function(WalletUnlockResult_ServerError value) serverError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletUnlockResult_Ok value)? ok,
    TResult? Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletUnlockResult_Timeout value)? timeout,
    TResult? Function(WalletUnlockResult_Blocked value)? blocked,
    TResult? Function(WalletUnlockResult_ServerError value)? serverError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletUnlockResult_Ok value)? ok,
    TResult Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletUnlockResult_Timeout value)? timeout,
    TResult Function(WalletUnlockResult_Blocked value)? blocked,
    TResult Function(WalletUnlockResult_ServerError value)? serverError,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $WalletUnlockResultCopyWith<$Res> {
  factory $WalletUnlockResultCopyWith(WalletUnlockResult value, $Res Function(WalletUnlockResult) then) =
      _$WalletUnlockResultCopyWithImpl<$Res, WalletUnlockResult>;
}

/// @nodoc
class _$WalletUnlockResultCopyWithImpl<$Res, $Val extends WalletUnlockResult>
    implements $WalletUnlockResultCopyWith<$Res> {
  _$WalletUnlockResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;
}

/// @nodoc
abstract class _$$WalletUnlockResult_OkCopyWith<$Res> {
  factory _$$WalletUnlockResult_OkCopyWith(_$WalletUnlockResult_Ok value, $Res Function(_$WalletUnlockResult_Ok) then) =
      __$$WalletUnlockResult_OkCopyWithImpl<$Res>;
}

/// @nodoc
class __$$WalletUnlockResult_OkCopyWithImpl<$Res>
    extends _$WalletUnlockResultCopyWithImpl<$Res, _$WalletUnlockResult_Ok>
    implements _$$WalletUnlockResult_OkCopyWith<$Res> {
  __$$WalletUnlockResult_OkCopyWithImpl(_$WalletUnlockResult_Ok _value, $Res Function(_$WalletUnlockResult_Ok) _then)
      : super(_value, _then);
}

/// @nodoc

class _$WalletUnlockResult_Ok implements WalletUnlockResult_Ok {
  const _$WalletUnlockResult_Ok();

  @override
  String toString() {
    return 'WalletUnlockResult.ok()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$WalletUnlockResult_Ok);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
    required TResult Function() serverError,
  }) {
    return ok();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
    TResult? Function()? serverError,
  }) {
    return ok?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    TResult Function()? serverError,
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
    required TResult Function(WalletUnlockResult_Ok value) ok,
    required TResult Function(WalletUnlockResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletUnlockResult_Timeout value) timeout,
    required TResult Function(WalletUnlockResult_Blocked value) blocked,
    required TResult Function(WalletUnlockResult_ServerError value) serverError,
  }) {
    return ok(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletUnlockResult_Ok value)? ok,
    TResult? Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletUnlockResult_Timeout value)? timeout,
    TResult? Function(WalletUnlockResult_Blocked value)? blocked,
    TResult? Function(WalletUnlockResult_ServerError value)? serverError,
  }) {
    return ok?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletUnlockResult_Ok value)? ok,
    TResult Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletUnlockResult_Timeout value)? timeout,
    TResult Function(WalletUnlockResult_Blocked value)? blocked,
    TResult Function(WalletUnlockResult_ServerError value)? serverError,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok(this);
    }
    return orElse();
  }
}

abstract class WalletUnlockResult_Ok implements WalletUnlockResult {
  const factory WalletUnlockResult_Ok() = _$WalletUnlockResult_Ok;
}

/// @nodoc
abstract class _$$WalletUnlockResult_IncorrectPinCopyWith<$Res> {
  factory _$$WalletUnlockResult_IncorrectPinCopyWith(
          _$WalletUnlockResult_IncorrectPin value, $Res Function(_$WalletUnlockResult_IncorrectPin) then) =
      __$$WalletUnlockResult_IncorrectPinCopyWithImpl<$Res>;
  @useResult
  $Res call({int leftoverAttempts, bool isFinalAttempt});
}

/// @nodoc
class __$$WalletUnlockResult_IncorrectPinCopyWithImpl<$Res>
    extends _$WalletUnlockResultCopyWithImpl<$Res, _$WalletUnlockResult_IncorrectPin>
    implements _$$WalletUnlockResult_IncorrectPinCopyWith<$Res> {
  __$$WalletUnlockResult_IncorrectPinCopyWithImpl(
      _$WalletUnlockResult_IncorrectPin _value, $Res Function(_$WalletUnlockResult_IncorrectPin) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? leftoverAttempts = null,
    Object? isFinalAttempt = null,
  }) {
    return _then(_$WalletUnlockResult_IncorrectPin(
      leftoverAttempts: null == leftoverAttempts
          ? _value.leftoverAttempts
          : leftoverAttempts // ignore: cast_nullable_to_non_nullable
              as int,
      isFinalAttempt: null == isFinalAttempt
          ? _value.isFinalAttempt
          : isFinalAttempt // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc

class _$WalletUnlockResult_IncorrectPin implements WalletUnlockResult_IncorrectPin {
  const _$WalletUnlockResult_IncorrectPin({required this.leftoverAttempts, required this.isFinalAttempt});

  @override
  final int leftoverAttempts;
  @override
  final bool isFinalAttempt;

  @override
  String toString() {
    return 'WalletUnlockResult.incorrectPin(leftoverAttempts: $leftoverAttempts, isFinalAttempt: $isFinalAttempt)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletUnlockResult_IncorrectPin &&
            (identical(other.leftoverAttempts, leftoverAttempts) || other.leftoverAttempts == leftoverAttempts) &&
            (identical(other.isFinalAttempt, isFinalAttempt) || other.isFinalAttempt == isFinalAttempt));
  }

  @override
  int get hashCode => Object.hash(runtimeType, leftoverAttempts, isFinalAttempt);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletUnlockResult_IncorrectPinCopyWith<_$WalletUnlockResult_IncorrectPin> get copyWith =>
      __$$WalletUnlockResult_IncorrectPinCopyWithImpl<_$WalletUnlockResult_IncorrectPin>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
    required TResult Function() serverError,
  }) {
    return incorrectPin(leftoverAttempts, isFinalAttempt);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
    TResult? Function()? serverError,
  }) {
    return incorrectPin?.call(leftoverAttempts, isFinalAttempt);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    TResult Function()? serverError,
    required TResult orElse(),
  }) {
    if (incorrectPin != null) {
      return incorrectPin(leftoverAttempts, isFinalAttempt);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletUnlockResult_Ok value) ok,
    required TResult Function(WalletUnlockResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletUnlockResult_Timeout value) timeout,
    required TResult Function(WalletUnlockResult_Blocked value) blocked,
    required TResult Function(WalletUnlockResult_ServerError value) serverError,
  }) {
    return incorrectPin(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletUnlockResult_Ok value)? ok,
    TResult? Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletUnlockResult_Timeout value)? timeout,
    TResult? Function(WalletUnlockResult_Blocked value)? blocked,
    TResult? Function(WalletUnlockResult_ServerError value)? serverError,
  }) {
    return incorrectPin?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletUnlockResult_Ok value)? ok,
    TResult Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletUnlockResult_Timeout value)? timeout,
    TResult Function(WalletUnlockResult_Blocked value)? blocked,
    TResult Function(WalletUnlockResult_ServerError value)? serverError,
    required TResult orElse(),
  }) {
    if (incorrectPin != null) {
      return incorrectPin(this);
    }
    return orElse();
  }
}

abstract class WalletUnlockResult_IncorrectPin implements WalletUnlockResult {
  const factory WalletUnlockResult_IncorrectPin(
      {required final int leftoverAttempts, required final bool isFinalAttempt}) = _$WalletUnlockResult_IncorrectPin;

  int get leftoverAttempts;
  bool get isFinalAttempt;
  @JsonKey(ignore: true)
  _$$WalletUnlockResult_IncorrectPinCopyWith<_$WalletUnlockResult_IncorrectPin> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletUnlockResult_TimeoutCopyWith<$Res> {
  factory _$$WalletUnlockResult_TimeoutCopyWith(
          _$WalletUnlockResult_Timeout value, $Res Function(_$WalletUnlockResult_Timeout) then) =
      __$$WalletUnlockResult_TimeoutCopyWithImpl<$Res>;
  @useResult
  $Res call({int timeoutMillis});
}

/// @nodoc
class __$$WalletUnlockResult_TimeoutCopyWithImpl<$Res>
    extends _$WalletUnlockResultCopyWithImpl<$Res, _$WalletUnlockResult_Timeout>
    implements _$$WalletUnlockResult_TimeoutCopyWith<$Res> {
  __$$WalletUnlockResult_TimeoutCopyWithImpl(
      _$WalletUnlockResult_Timeout _value, $Res Function(_$WalletUnlockResult_Timeout) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timeoutMillis = null,
  }) {
    return _then(_$WalletUnlockResult_Timeout(
      timeoutMillis: null == timeoutMillis
          ? _value.timeoutMillis
          : timeoutMillis // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$WalletUnlockResult_Timeout implements WalletUnlockResult_Timeout {
  const _$WalletUnlockResult_Timeout({required this.timeoutMillis});

  @override
  final int timeoutMillis;

  @override
  String toString() {
    return 'WalletUnlockResult.timeout(timeoutMillis: $timeoutMillis)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletUnlockResult_Timeout &&
            (identical(other.timeoutMillis, timeoutMillis) || other.timeoutMillis == timeoutMillis));
  }

  @override
  int get hashCode => Object.hash(runtimeType, timeoutMillis);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletUnlockResult_TimeoutCopyWith<_$WalletUnlockResult_Timeout> get copyWith =>
      __$$WalletUnlockResult_TimeoutCopyWithImpl<_$WalletUnlockResult_Timeout>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
    required TResult Function() serverError,
  }) {
    return timeout(timeoutMillis);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
    TResult? Function()? serverError,
  }) {
    return timeout?.call(timeoutMillis);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    TResult Function()? serverError,
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
    required TResult Function(WalletUnlockResult_Ok value) ok,
    required TResult Function(WalletUnlockResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletUnlockResult_Timeout value) timeout,
    required TResult Function(WalletUnlockResult_Blocked value) blocked,
    required TResult Function(WalletUnlockResult_ServerError value) serverError,
  }) {
    return timeout(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletUnlockResult_Ok value)? ok,
    TResult? Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletUnlockResult_Timeout value)? timeout,
    TResult? Function(WalletUnlockResult_Blocked value)? blocked,
    TResult? Function(WalletUnlockResult_ServerError value)? serverError,
  }) {
    return timeout?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletUnlockResult_Ok value)? ok,
    TResult Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletUnlockResult_Timeout value)? timeout,
    TResult Function(WalletUnlockResult_Blocked value)? blocked,
    TResult Function(WalletUnlockResult_ServerError value)? serverError,
    required TResult orElse(),
  }) {
    if (timeout != null) {
      return timeout(this);
    }
    return orElse();
  }
}

abstract class WalletUnlockResult_Timeout implements WalletUnlockResult {
  const factory WalletUnlockResult_Timeout({required final int timeoutMillis}) = _$WalletUnlockResult_Timeout;

  int get timeoutMillis;
  @JsonKey(ignore: true)
  _$$WalletUnlockResult_TimeoutCopyWith<_$WalletUnlockResult_Timeout> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletUnlockResult_BlockedCopyWith<$Res> {
  factory _$$WalletUnlockResult_BlockedCopyWith(
          _$WalletUnlockResult_Blocked value, $Res Function(_$WalletUnlockResult_Blocked) then) =
      __$$WalletUnlockResult_BlockedCopyWithImpl<$Res>;
}

/// @nodoc
class __$$WalletUnlockResult_BlockedCopyWithImpl<$Res>
    extends _$WalletUnlockResultCopyWithImpl<$Res, _$WalletUnlockResult_Blocked>
    implements _$$WalletUnlockResult_BlockedCopyWith<$Res> {
  __$$WalletUnlockResult_BlockedCopyWithImpl(
      _$WalletUnlockResult_Blocked _value, $Res Function(_$WalletUnlockResult_Blocked) _then)
      : super(_value, _then);
}

/// @nodoc

class _$WalletUnlockResult_Blocked implements WalletUnlockResult_Blocked {
  const _$WalletUnlockResult_Blocked();

  @override
  String toString() {
    return 'WalletUnlockResult.blocked()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$WalletUnlockResult_Blocked);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
    required TResult Function() serverError,
  }) {
    return blocked();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
    TResult? Function()? serverError,
  }) {
    return blocked?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    TResult Function()? serverError,
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
    required TResult Function(WalletUnlockResult_Ok value) ok,
    required TResult Function(WalletUnlockResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletUnlockResult_Timeout value) timeout,
    required TResult Function(WalletUnlockResult_Blocked value) blocked,
    required TResult Function(WalletUnlockResult_ServerError value) serverError,
  }) {
    return blocked(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletUnlockResult_Ok value)? ok,
    TResult? Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletUnlockResult_Timeout value)? timeout,
    TResult? Function(WalletUnlockResult_Blocked value)? blocked,
    TResult? Function(WalletUnlockResult_ServerError value)? serverError,
  }) {
    return blocked?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletUnlockResult_Ok value)? ok,
    TResult Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletUnlockResult_Timeout value)? timeout,
    TResult Function(WalletUnlockResult_Blocked value)? blocked,
    TResult Function(WalletUnlockResult_ServerError value)? serverError,
    required TResult orElse(),
  }) {
    if (blocked != null) {
      return blocked(this);
    }
    return orElse();
  }
}

abstract class WalletUnlockResult_Blocked implements WalletUnlockResult {
  const factory WalletUnlockResult_Blocked() = _$WalletUnlockResult_Blocked;
}

/// @nodoc
abstract class _$$WalletUnlockResult_ServerErrorCopyWith<$Res> {
  factory _$$WalletUnlockResult_ServerErrorCopyWith(
          _$WalletUnlockResult_ServerError value, $Res Function(_$WalletUnlockResult_ServerError) then) =
      __$$WalletUnlockResult_ServerErrorCopyWithImpl<$Res>;
}

/// @nodoc
class __$$WalletUnlockResult_ServerErrorCopyWithImpl<$Res>
    extends _$WalletUnlockResultCopyWithImpl<$Res, _$WalletUnlockResult_ServerError>
    implements _$$WalletUnlockResult_ServerErrorCopyWith<$Res> {
  __$$WalletUnlockResult_ServerErrorCopyWithImpl(
      _$WalletUnlockResult_ServerError _value, $Res Function(_$WalletUnlockResult_ServerError) _then)
      : super(_value, _then);
}

/// @nodoc

class _$WalletUnlockResult_ServerError implements WalletUnlockResult_ServerError {
  const _$WalletUnlockResult_ServerError();

  @override
  String toString() {
    return 'WalletUnlockResult.serverError()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$WalletUnlockResult_ServerError);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
    required TResult Function() serverError,
  }) {
    return serverError();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
    TResult? Function()? serverError,
  }) {
    return serverError?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    TResult Function()? serverError,
    required TResult orElse(),
  }) {
    if (serverError != null) {
      return serverError();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletUnlockResult_Ok value) ok,
    required TResult Function(WalletUnlockResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletUnlockResult_Timeout value) timeout,
    required TResult Function(WalletUnlockResult_Blocked value) blocked,
    required TResult Function(WalletUnlockResult_ServerError value) serverError,
  }) {
    return serverError(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletUnlockResult_Ok value)? ok,
    TResult? Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletUnlockResult_Timeout value)? timeout,
    TResult? Function(WalletUnlockResult_Blocked value)? blocked,
    TResult? Function(WalletUnlockResult_ServerError value)? serverError,
  }) {
    return serverError?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletUnlockResult_Ok value)? ok,
    TResult Function(WalletUnlockResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletUnlockResult_Timeout value)? timeout,
    TResult Function(WalletUnlockResult_Blocked value)? blocked,
    TResult Function(WalletUnlockResult_ServerError value)? serverError,
    required TResult orElse(),
  }) {
    if (serverError != null) {
      return serverError(this);
    }
    return orElse();
  }
}

abstract class WalletUnlockResult_ServerError implements WalletUnlockResult {
  const factory WalletUnlockResult_ServerError() = _$WalletUnlockResult_ServerError;
}
