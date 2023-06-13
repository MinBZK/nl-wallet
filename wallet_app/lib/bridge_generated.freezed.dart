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
