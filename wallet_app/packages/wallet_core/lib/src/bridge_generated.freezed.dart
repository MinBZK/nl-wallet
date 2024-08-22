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
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$AcceptDisclosureResult {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String? returnUrl) ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String? returnUrl)? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String? returnUrl)? ok,
    TResult Function(WalletInstructionError error)? instructionError,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AcceptDisclosureResult_Ok value) ok,
    required TResult Function(AcceptDisclosureResult_InstructionError value) instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AcceptDisclosureResult_Ok value)? ok,
    TResult? Function(AcceptDisclosureResult_InstructionError value)? instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AcceptDisclosureResult_Ok value)? ok,
    TResult Function(AcceptDisclosureResult_InstructionError value)? instructionError,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AcceptDisclosureResultCopyWith<$Res> {
  factory $AcceptDisclosureResultCopyWith(AcceptDisclosureResult value, $Res Function(AcceptDisclosureResult) then) =
      _$AcceptDisclosureResultCopyWithImpl<$Res, AcceptDisclosureResult>;
}

/// @nodoc
class _$AcceptDisclosureResultCopyWithImpl<$Res, $Val extends AcceptDisclosureResult>
    implements $AcceptDisclosureResultCopyWith<$Res> {
  _$AcceptDisclosureResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$AcceptDisclosureResult_OkImplCopyWith<$Res> {
  factory _$$AcceptDisclosureResult_OkImplCopyWith(
          _$AcceptDisclosureResult_OkImpl value, $Res Function(_$AcceptDisclosureResult_OkImpl) then) =
      __$$AcceptDisclosureResult_OkImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String? returnUrl});
}

/// @nodoc
class __$$AcceptDisclosureResult_OkImplCopyWithImpl<$Res>
    extends _$AcceptDisclosureResultCopyWithImpl<$Res, _$AcceptDisclosureResult_OkImpl>
    implements _$$AcceptDisclosureResult_OkImplCopyWith<$Res> {
  __$$AcceptDisclosureResult_OkImplCopyWithImpl(
      _$AcceptDisclosureResult_OkImpl _value, $Res Function(_$AcceptDisclosureResult_OkImpl) _then)
      : super(_value, _then);

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? returnUrl = freezed,
  }) {
    return _then(_$AcceptDisclosureResult_OkImpl(
      returnUrl: freezed == returnUrl
          ? _value.returnUrl
          : returnUrl // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc

class _$AcceptDisclosureResult_OkImpl implements AcceptDisclosureResult_Ok {
  const _$AcceptDisclosureResult_OkImpl({this.returnUrl});

  @override
  final String? returnUrl;

  @override
  String toString() {
    return 'AcceptDisclosureResult.ok(returnUrl: $returnUrl)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AcceptDisclosureResult_OkImpl &&
            (identical(other.returnUrl, returnUrl) || other.returnUrl == returnUrl));
  }

  @override
  int get hashCode => Object.hash(runtimeType, returnUrl);

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AcceptDisclosureResult_OkImplCopyWith<_$AcceptDisclosureResult_OkImpl> get copyWith =>
      __$$AcceptDisclosureResult_OkImplCopyWithImpl<_$AcceptDisclosureResult_OkImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String? returnUrl) ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) {
    return ok(returnUrl);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String? returnUrl)? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) {
    return ok?.call(returnUrl);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String? returnUrl)? ok,
    TResult Function(WalletInstructionError error)? instructionError,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok(returnUrl);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AcceptDisclosureResult_Ok value) ok,
    required TResult Function(AcceptDisclosureResult_InstructionError value) instructionError,
  }) {
    return ok(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AcceptDisclosureResult_Ok value)? ok,
    TResult? Function(AcceptDisclosureResult_InstructionError value)? instructionError,
  }) {
    return ok?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AcceptDisclosureResult_Ok value)? ok,
    TResult Function(AcceptDisclosureResult_InstructionError value)? instructionError,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok(this);
    }
    return orElse();
  }
}

abstract class AcceptDisclosureResult_Ok implements AcceptDisclosureResult {
  const factory AcceptDisclosureResult_Ok({final String? returnUrl}) = _$AcceptDisclosureResult_OkImpl;

  String? get returnUrl;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AcceptDisclosureResult_OkImplCopyWith<_$AcceptDisclosureResult_OkImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$AcceptDisclosureResult_InstructionErrorImplCopyWith<$Res> {
  factory _$$AcceptDisclosureResult_InstructionErrorImplCopyWith(_$AcceptDisclosureResult_InstructionErrorImpl value,
          $Res Function(_$AcceptDisclosureResult_InstructionErrorImpl) then) =
      __$$AcceptDisclosureResult_InstructionErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({WalletInstructionError error});

  $WalletInstructionErrorCopyWith<$Res> get error;
}

/// @nodoc
class __$$AcceptDisclosureResult_InstructionErrorImplCopyWithImpl<$Res>
    extends _$AcceptDisclosureResultCopyWithImpl<$Res, _$AcceptDisclosureResult_InstructionErrorImpl>
    implements _$$AcceptDisclosureResult_InstructionErrorImplCopyWith<$Res> {
  __$$AcceptDisclosureResult_InstructionErrorImplCopyWithImpl(_$AcceptDisclosureResult_InstructionErrorImpl _value,
      $Res Function(_$AcceptDisclosureResult_InstructionErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? error = null,
  }) {
    return _then(_$AcceptDisclosureResult_InstructionErrorImpl(
      error: null == error
          ? _value.error
          : error // ignore: cast_nullable_to_non_nullable
              as WalletInstructionError,
    ));
  }

  /// Create a copy of AcceptDisclosureResult
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

class _$AcceptDisclosureResult_InstructionErrorImpl implements AcceptDisclosureResult_InstructionError {
  const _$AcceptDisclosureResult_InstructionErrorImpl({required this.error});

  @override
  final WalletInstructionError error;

  @override
  String toString() {
    return 'AcceptDisclosureResult.instructionError(error: $error)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AcceptDisclosureResult_InstructionErrorImpl &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AcceptDisclosureResult_InstructionErrorImplCopyWith<_$AcceptDisclosureResult_InstructionErrorImpl> get copyWith =>
      __$$AcceptDisclosureResult_InstructionErrorImplCopyWithImpl<_$AcceptDisclosureResult_InstructionErrorImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String? returnUrl) ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) {
    return instructionError(error);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String? returnUrl)? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) {
    return instructionError?.call(error);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String? returnUrl)? ok,
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
    required TResult Function(AcceptDisclosureResult_Ok value) ok,
    required TResult Function(AcceptDisclosureResult_InstructionError value) instructionError,
  }) {
    return instructionError(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AcceptDisclosureResult_Ok value)? ok,
    TResult? Function(AcceptDisclosureResult_InstructionError value)? instructionError,
  }) {
    return instructionError?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AcceptDisclosureResult_Ok value)? ok,
    TResult Function(AcceptDisclosureResult_InstructionError value)? instructionError,
    required TResult orElse(),
  }) {
    if (instructionError != null) {
      return instructionError(this);
    }
    return orElse();
  }
}

abstract class AcceptDisclosureResult_InstructionError implements AcceptDisclosureResult {
  const factory AcceptDisclosureResult_InstructionError({required final WalletInstructionError error}) =
      _$AcceptDisclosureResult_InstructionErrorImpl;

  WalletInstructionError get error;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AcceptDisclosureResult_InstructionErrorImplCopyWith<_$AcceptDisclosureResult_InstructionErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$CardPersistence {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() inMemory,
    required TResult Function(String id) stored,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? inMemory,
    TResult? Function(String id)? stored,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? inMemory,
    TResult Function(String id)? stored,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(CardPersistence_InMemory value) inMemory,
    required TResult Function(CardPersistence_Stored value) stored,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(CardPersistence_InMemory value)? inMemory,
    TResult? Function(CardPersistence_Stored value)? stored,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(CardPersistence_InMemory value)? inMemory,
    TResult Function(CardPersistence_Stored value)? stored,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CardPersistenceCopyWith<$Res> {
  factory $CardPersistenceCopyWith(CardPersistence value, $Res Function(CardPersistence) then) =
      _$CardPersistenceCopyWithImpl<$Res, CardPersistence>;
}

/// @nodoc
class _$CardPersistenceCopyWithImpl<$Res, $Val extends CardPersistence> implements $CardPersistenceCopyWith<$Res> {
  _$CardPersistenceCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CardPersistence
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$CardPersistence_InMemoryImplCopyWith<$Res> {
  factory _$$CardPersistence_InMemoryImplCopyWith(
          _$CardPersistence_InMemoryImpl value, $Res Function(_$CardPersistence_InMemoryImpl) then) =
      __$$CardPersistence_InMemoryImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$CardPersistence_InMemoryImplCopyWithImpl<$Res>
    extends _$CardPersistenceCopyWithImpl<$Res, _$CardPersistence_InMemoryImpl>
    implements _$$CardPersistence_InMemoryImplCopyWith<$Res> {
  __$$CardPersistence_InMemoryImplCopyWithImpl(
      _$CardPersistence_InMemoryImpl _value, $Res Function(_$CardPersistence_InMemoryImpl) _then)
      : super(_value, _then);

  /// Create a copy of CardPersistence
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$CardPersistence_InMemoryImpl implements CardPersistence_InMemory {
  const _$CardPersistence_InMemoryImpl();

  @override
  String toString() {
    return 'CardPersistence.inMemory()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$CardPersistence_InMemoryImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() inMemory,
    required TResult Function(String id) stored,
  }) {
    return inMemory();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? inMemory,
    TResult? Function(String id)? stored,
  }) {
    return inMemory?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? inMemory,
    TResult Function(String id)? stored,
    required TResult orElse(),
  }) {
    if (inMemory != null) {
      return inMemory();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(CardPersistence_InMemory value) inMemory,
    required TResult Function(CardPersistence_Stored value) stored,
  }) {
    return inMemory(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(CardPersistence_InMemory value)? inMemory,
    TResult? Function(CardPersistence_Stored value)? stored,
  }) {
    return inMemory?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(CardPersistence_InMemory value)? inMemory,
    TResult Function(CardPersistence_Stored value)? stored,
    required TResult orElse(),
  }) {
    if (inMemory != null) {
      return inMemory(this);
    }
    return orElse();
  }
}

abstract class CardPersistence_InMemory implements CardPersistence {
  const factory CardPersistence_InMemory() = _$CardPersistence_InMemoryImpl;
}

/// @nodoc
abstract class _$$CardPersistence_StoredImplCopyWith<$Res> {
  factory _$$CardPersistence_StoredImplCopyWith(
          _$CardPersistence_StoredImpl value, $Res Function(_$CardPersistence_StoredImpl) then) =
      __$$CardPersistence_StoredImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String id});
}

/// @nodoc
class __$$CardPersistence_StoredImplCopyWithImpl<$Res>
    extends _$CardPersistenceCopyWithImpl<$Res, _$CardPersistence_StoredImpl>
    implements _$$CardPersistence_StoredImplCopyWith<$Res> {
  __$$CardPersistence_StoredImplCopyWithImpl(
      _$CardPersistence_StoredImpl _value, $Res Function(_$CardPersistence_StoredImpl) _then)
      : super(_value, _then);

  /// Create a copy of CardPersistence
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
  }) {
    return _then(_$CardPersistence_StoredImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$CardPersistence_StoredImpl implements CardPersistence_Stored {
  const _$CardPersistence_StoredImpl({required this.id});

  @override
  final String id;

  @override
  String toString() {
    return 'CardPersistence.stored(id: $id)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardPersistence_StoredImpl &&
            (identical(other.id, id) || other.id == id));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id);

  /// Create a copy of CardPersistence
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CardPersistence_StoredImplCopyWith<_$CardPersistence_StoredImpl> get copyWith =>
      __$$CardPersistence_StoredImplCopyWithImpl<_$CardPersistence_StoredImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() inMemory,
    required TResult Function(String id) stored,
  }) {
    return stored(id);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? inMemory,
    TResult? Function(String id)? stored,
  }) {
    return stored?.call(id);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? inMemory,
    TResult Function(String id)? stored,
    required TResult orElse(),
  }) {
    if (stored != null) {
      return stored(id);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(CardPersistence_InMemory value) inMemory,
    required TResult Function(CardPersistence_Stored value) stored,
  }) {
    return stored(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(CardPersistence_InMemory value)? inMemory,
    TResult? Function(CardPersistence_Stored value)? stored,
  }) {
    return stored?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(CardPersistence_InMemory value)? inMemory,
    TResult Function(CardPersistence_Stored value)? stored,
    required TResult orElse(),
  }) {
    if (stored != null) {
      return stored(this);
    }
    return orElse();
  }
}

abstract class CardPersistence_Stored implements CardPersistence {
  const factory CardPersistence_Stored({required final String id}) = _$CardPersistence_StoredImpl;

  String get id;

  /// Create a copy of CardPersistence
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CardPersistence_StoredImplCopyWith<_$CardPersistence_StoredImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$CardValue {
  Object get value => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String value) string,
    required TResult Function(bool value) boolean,
    required TResult Function(String value) date,
    required TResult Function(GenderCardValue value) gender,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String value)? string,
    TResult? Function(bool value)? boolean,
    TResult? Function(String value)? date,
    TResult? Function(GenderCardValue value)? gender,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String value)? string,
    TResult Function(bool value)? boolean,
    TResult Function(String value)? date,
    TResult Function(GenderCardValue value)? gender,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(CardValue_String value) string,
    required TResult Function(CardValue_Boolean value) boolean,
    required TResult Function(CardValue_Date value) date,
    required TResult Function(CardValue_Gender value) gender,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(CardValue_String value)? string,
    TResult? Function(CardValue_Boolean value)? boolean,
    TResult? Function(CardValue_Date value)? date,
    TResult? Function(CardValue_Gender value)? gender,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(CardValue_String value)? string,
    TResult Function(CardValue_Boolean value)? boolean,
    TResult Function(CardValue_Date value)? date,
    TResult Function(CardValue_Gender value)? gender,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CardValueCopyWith<$Res> {
  factory $CardValueCopyWith(CardValue value, $Res Function(CardValue) then) = _$CardValueCopyWithImpl<$Res, CardValue>;
}

/// @nodoc
class _$CardValueCopyWithImpl<$Res, $Val extends CardValue> implements $CardValueCopyWith<$Res> {
  _$CardValueCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$CardValue_StringImplCopyWith<$Res> {
  factory _$$CardValue_StringImplCopyWith(_$CardValue_StringImpl value, $Res Function(_$CardValue_StringImpl) then) =
      __$$CardValue_StringImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String value});
}

/// @nodoc
class __$$CardValue_StringImplCopyWithImpl<$Res> extends _$CardValueCopyWithImpl<$Res, _$CardValue_StringImpl>
    implements _$$CardValue_StringImplCopyWith<$Res> {
  __$$CardValue_StringImplCopyWithImpl(_$CardValue_StringImpl _value, $Res Function(_$CardValue_StringImpl) _then)
      : super(_value, _then);

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$CardValue_StringImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$CardValue_StringImpl implements CardValue_String {
  const _$CardValue_StringImpl({required this.value});

  @override
  final String value;

  @override
  String toString() {
    return 'CardValue.string(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardValue_StringImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CardValue_StringImplCopyWith<_$CardValue_StringImpl> get copyWith =>
      __$$CardValue_StringImplCopyWithImpl<_$CardValue_StringImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String value) string,
    required TResult Function(bool value) boolean,
    required TResult Function(String value) date,
    required TResult Function(GenderCardValue value) gender,
  }) {
    return string(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String value)? string,
    TResult? Function(bool value)? boolean,
    TResult? Function(String value)? date,
    TResult? Function(GenderCardValue value)? gender,
  }) {
    return string?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String value)? string,
    TResult Function(bool value)? boolean,
    TResult Function(String value)? date,
    TResult Function(GenderCardValue value)? gender,
    required TResult orElse(),
  }) {
    if (string != null) {
      return string(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(CardValue_String value) string,
    required TResult Function(CardValue_Boolean value) boolean,
    required TResult Function(CardValue_Date value) date,
    required TResult Function(CardValue_Gender value) gender,
  }) {
    return string(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(CardValue_String value)? string,
    TResult? Function(CardValue_Boolean value)? boolean,
    TResult? Function(CardValue_Date value)? date,
    TResult? Function(CardValue_Gender value)? gender,
  }) {
    return string?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(CardValue_String value)? string,
    TResult Function(CardValue_Boolean value)? boolean,
    TResult Function(CardValue_Date value)? date,
    TResult Function(CardValue_Gender value)? gender,
    required TResult orElse(),
  }) {
    if (string != null) {
      return string(this);
    }
    return orElse();
  }
}

abstract class CardValue_String implements CardValue {
  const factory CardValue_String({required final String value}) = _$CardValue_StringImpl;

  @override
  String get value;

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CardValue_StringImplCopyWith<_$CardValue_StringImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$CardValue_BooleanImplCopyWith<$Res> {
  factory _$$CardValue_BooleanImplCopyWith(_$CardValue_BooleanImpl value, $Res Function(_$CardValue_BooleanImpl) then) =
      __$$CardValue_BooleanImplCopyWithImpl<$Res>;
  @useResult
  $Res call({bool value});
}

/// @nodoc
class __$$CardValue_BooleanImplCopyWithImpl<$Res> extends _$CardValueCopyWithImpl<$Res, _$CardValue_BooleanImpl>
    implements _$$CardValue_BooleanImplCopyWith<$Res> {
  __$$CardValue_BooleanImplCopyWithImpl(_$CardValue_BooleanImpl _value, $Res Function(_$CardValue_BooleanImpl) _then)
      : super(_value, _then);

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$CardValue_BooleanImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc

class _$CardValue_BooleanImpl implements CardValue_Boolean {
  const _$CardValue_BooleanImpl({required this.value});

  @override
  final bool value;

  @override
  String toString() {
    return 'CardValue.boolean(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardValue_BooleanImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CardValue_BooleanImplCopyWith<_$CardValue_BooleanImpl> get copyWith =>
      __$$CardValue_BooleanImplCopyWithImpl<_$CardValue_BooleanImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String value) string,
    required TResult Function(bool value) boolean,
    required TResult Function(String value) date,
    required TResult Function(GenderCardValue value) gender,
  }) {
    return boolean(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String value)? string,
    TResult? Function(bool value)? boolean,
    TResult? Function(String value)? date,
    TResult? Function(GenderCardValue value)? gender,
  }) {
    return boolean?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String value)? string,
    TResult Function(bool value)? boolean,
    TResult Function(String value)? date,
    TResult Function(GenderCardValue value)? gender,
    required TResult orElse(),
  }) {
    if (boolean != null) {
      return boolean(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(CardValue_String value) string,
    required TResult Function(CardValue_Boolean value) boolean,
    required TResult Function(CardValue_Date value) date,
    required TResult Function(CardValue_Gender value) gender,
  }) {
    return boolean(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(CardValue_String value)? string,
    TResult? Function(CardValue_Boolean value)? boolean,
    TResult? Function(CardValue_Date value)? date,
    TResult? Function(CardValue_Gender value)? gender,
  }) {
    return boolean?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(CardValue_String value)? string,
    TResult Function(CardValue_Boolean value)? boolean,
    TResult Function(CardValue_Date value)? date,
    TResult Function(CardValue_Gender value)? gender,
    required TResult orElse(),
  }) {
    if (boolean != null) {
      return boolean(this);
    }
    return orElse();
  }
}

abstract class CardValue_Boolean implements CardValue {
  const factory CardValue_Boolean({required final bool value}) = _$CardValue_BooleanImpl;

  @override
  bool get value;

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CardValue_BooleanImplCopyWith<_$CardValue_BooleanImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$CardValue_DateImplCopyWith<$Res> {
  factory _$$CardValue_DateImplCopyWith(_$CardValue_DateImpl value, $Res Function(_$CardValue_DateImpl) then) =
      __$$CardValue_DateImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String value});
}

/// @nodoc
class __$$CardValue_DateImplCopyWithImpl<$Res> extends _$CardValueCopyWithImpl<$Res, _$CardValue_DateImpl>
    implements _$$CardValue_DateImplCopyWith<$Res> {
  __$$CardValue_DateImplCopyWithImpl(_$CardValue_DateImpl _value, $Res Function(_$CardValue_DateImpl) _then)
      : super(_value, _then);

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$CardValue_DateImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$CardValue_DateImpl implements CardValue_Date {
  const _$CardValue_DateImpl({required this.value});

  @override
  final String value;

  @override
  String toString() {
    return 'CardValue.date(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardValue_DateImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CardValue_DateImplCopyWith<_$CardValue_DateImpl> get copyWith =>
      __$$CardValue_DateImplCopyWithImpl<_$CardValue_DateImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String value) string,
    required TResult Function(bool value) boolean,
    required TResult Function(String value) date,
    required TResult Function(GenderCardValue value) gender,
  }) {
    return date(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String value)? string,
    TResult? Function(bool value)? boolean,
    TResult? Function(String value)? date,
    TResult? Function(GenderCardValue value)? gender,
  }) {
    return date?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String value)? string,
    TResult Function(bool value)? boolean,
    TResult Function(String value)? date,
    TResult Function(GenderCardValue value)? gender,
    required TResult orElse(),
  }) {
    if (date != null) {
      return date(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(CardValue_String value) string,
    required TResult Function(CardValue_Boolean value) boolean,
    required TResult Function(CardValue_Date value) date,
    required TResult Function(CardValue_Gender value) gender,
  }) {
    return date(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(CardValue_String value)? string,
    TResult? Function(CardValue_Boolean value)? boolean,
    TResult? Function(CardValue_Date value)? date,
    TResult? Function(CardValue_Gender value)? gender,
  }) {
    return date?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(CardValue_String value)? string,
    TResult Function(CardValue_Boolean value)? boolean,
    TResult Function(CardValue_Date value)? date,
    TResult Function(CardValue_Gender value)? gender,
    required TResult orElse(),
  }) {
    if (date != null) {
      return date(this);
    }
    return orElse();
  }
}

abstract class CardValue_Date implements CardValue {
  const factory CardValue_Date({required final String value}) = _$CardValue_DateImpl;

  @override
  String get value;

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CardValue_DateImplCopyWith<_$CardValue_DateImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$CardValue_GenderImplCopyWith<$Res> {
  factory _$$CardValue_GenderImplCopyWith(_$CardValue_GenderImpl value, $Res Function(_$CardValue_GenderImpl) then) =
      __$$CardValue_GenderImplCopyWithImpl<$Res>;
  @useResult
  $Res call({GenderCardValue value});
}

/// @nodoc
class __$$CardValue_GenderImplCopyWithImpl<$Res> extends _$CardValueCopyWithImpl<$Res, _$CardValue_GenderImpl>
    implements _$$CardValue_GenderImplCopyWith<$Res> {
  __$$CardValue_GenderImplCopyWithImpl(_$CardValue_GenderImpl _value, $Res Function(_$CardValue_GenderImpl) _then)
      : super(_value, _then);

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$CardValue_GenderImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as GenderCardValue,
    ));
  }
}

/// @nodoc

class _$CardValue_GenderImpl implements CardValue_Gender {
  const _$CardValue_GenderImpl({required this.value});

  @override
  final GenderCardValue value;

  @override
  String toString() {
    return 'CardValue.gender(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardValue_GenderImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CardValue_GenderImplCopyWith<_$CardValue_GenderImpl> get copyWith =>
      __$$CardValue_GenderImplCopyWithImpl<_$CardValue_GenderImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String value) string,
    required TResult Function(bool value) boolean,
    required TResult Function(String value) date,
    required TResult Function(GenderCardValue value) gender,
  }) {
    return gender(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String value)? string,
    TResult? Function(bool value)? boolean,
    TResult? Function(String value)? date,
    TResult? Function(GenderCardValue value)? gender,
  }) {
    return gender?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String value)? string,
    TResult Function(bool value)? boolean,
    TResult Function(String value)? date,
    TResult Function(GenderCardValue value)? gender,
    required TResult orElse(),
  }) {
    if (gender != null) {
      return gender(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(CardValue_String value) string,
    required TResult Function(CardValue_Boolean value) boolean,
    required TResult Function(CardValue_Date value) date,
    required TResult Function(CardValue_Gender value) gender,
  }) {
    return gender(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(CardValue_String value)? string,
    TResult? Function(CardValue_Boolean value)? boolean,
    TResult? Function(CardValue_Date value)? date,
    TResult? Function(CardValue_Gender value)? gender,
  }) {
    return gender?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(CardValue_String value)? string,
    TResult Function(CardValue_Boolean value)? boolean,
    TResult Function(CardValue_Date value)? date,
    TResult Function(CardValue_Gender value)? gender,
    required TResult orElse(),
  }) {
    if (gender != null) {
      return gender(this);
    }
    return orElse();
  }
}

abstract class CardValue_Gender implements CardValue {
  const factory CardValue_Gender({required final GenderCardValue value}) = _$CardValue_GenderImpl;

  @override
  GenderCardValue get value;

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CardValue_GenderImplCopyWith<_$CardValue_GenderImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$Image {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ImageCopyWith<$Res> {
  factory $ImageCopyWith(Image value, $Res Function(Image) then) = _$ImageCopyWithImpl<$Res, Image>;
}

/// @nodoc
class _$ImageCopyWithImpl<$Res, $Val extends Image> implements $ImageCopyWith<$Res> {
  _$ImageCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$Image_SvgImplCopyWith<$Res> {
  factory _$$Image_SvgImplCopyWith(_$Image_SvgImpl value, $Res Function(_$Image_SvgImpl) then) =
      __$$Image_SvgImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String xml});
}

/// @nodoc
class __$$Image_SvgImplCopyWithImpl<$Res> extends _$ImageCopyWithImpl<$Res, _$Image_SvgImpl>
    implements _$$Image_SvgImplCopyWith<$Res> {
  __$$Image_SvgImplCopyWithImpl(_$Image_SvgImpl _value, $Res Function(_$Image_SvgImpl) _then) : super(_value, _then);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? xml = null,
  }) {
    return _then(_$Image_SvgImpl(
      xml: null == xml
          ? _value.xml
          : xml // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$Image_SvgImpl implements Image_Svg {
  const _$Image_SvgImpl({required this.xml});

  @override
  final String xml;

  @override
  String toString() {
    return 'Image.svg(xml: $xml)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Image_SvgImpl &&
            (identical(other.xml, xml) || other.xml == xml));
  }

  @override
  int get hashCode => Object.hash(runtimeType, xml);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Image_SvgImplCopyWith<_$Image_SvgImpl> get copyWith =>
      __$$Image_SvgImplCopyWithImpl<_$Image_SvgImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) {
    return svg(xml);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) {
    return svg?.call(xml);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    if (svg != null) {
      return svg(xml);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) {
    return svg(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    return svg?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    if (svg != null) {
      return svg(this);
    }
    return orElse();
  }
}

abstract class Image_Svg implements Image {
  const factory Image_Svg({required final String xml}) = _$Image_SvgImpl;

  String get xml;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Image_SvgImplCopyWith<_$Image_SvgImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Image_PngImplCopyWith<$Res> {
  factory _$$Image_PngImplCopyWith(_$Image_PngImpl value, $Res Function(_$Image_PngImpl) then) =
      __$$Image_PngImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String base64});
}

/// @nodoc
class __$$Image_PngImplCopyWithImpl<$Res> extends _$ImageCopyWithImpl<$Res, _$Image_PngImpl>
    implements _$$Image_PngImplCopyWith<$Res> {
  __$$Image_PngImplCopyWithImpl(_$Image_PngImpl _value, $Res Function(_$Image_PngImpl) _then) : super(_value, _then);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? base64 = null,
  }) {
    return _then(_$Image_PngImpl(
      base64: null == base64
          ? _value.base64
          : base64 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$Image_PngImpl implements Image_Png {
  const _$Image_PngImpl({required this.base64});

  @override
  final String base64;

  @override
  String toString() {
    return 'Image.png(base64: $base64)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Image_PngImpl &&
            (identical(other.base64, base64) || other.base64 == base64));
  }

  @override
  int get hashCode => Object.hash(runtimeType, base64);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Image_PngImplCopyWith<_$Image_PngImpl> get copyWith =>
      __$$Image_PngImplCopyWithImpl<_$Image_PngImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) {
    return png(base64);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) {
    return png?.call(base64);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    if (png != null) {
      return png(base64);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) {
    return png(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    return png?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    if (png != null) {
      return png(this);
    }
    return orElse();
  }
}

abstract class Image_Png implements Image {
  const factory Image_Png({required final String base64}) = _$Image_PngImpl;

  String get base64;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Image_PngImplCopyWith<_$Image_PngImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Image_JpgImplCopyWith<$Res> {
  factory _$$Image_JpgImplCopyWith(_$Image_JpgImpl value, $Res Function(_$Image_JpgImpl) then) =
      __$$Image_JpgImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String base64});
}

/// @nodoc
class __$$Image_JpgImplCopyWithImpl<$Res> extends _$ImageCopyWithImpl<$Res, _$Image_JpgImpl>
    implements _$$Image_JpgImplCopyWith<$Res> {
  __$$Image_JpgImplCopyWithImpl(_$Image_JpgImpl _value, $Res Function(_$Image_JpgImpl) _then) : super(_value, _then);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? base64 = null,
  }) {
    return _then(_$Image_JpgImpl(
      base64: null == base64
          ? _value.base64
          : base64 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$Image_JpgImpl implements Image_Jpg {
  const _$Image_JpgImpl({required this.base64});

  @override
  final String base64;

  @override
  String toString() {
    return 'Image.jpg(base64: $base64)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Image_JpgImpl &&
            (identical(other.base64, base64) || other.base64 == base64));
  }

  @override
  int get hashCode => Object.hash(runtimeType, base64);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Image_JpgImplCopyWith<_$Image_JpgImpl> get copyWith =>
      __$$Image_JpgImplCopyWithImpl<_$Image_JpgImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) {
    return jpg(base64);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) {
    return jpg?.call(base64);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    if (jpg != null) {
      return jpg(base64);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) {
    return jpg(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    return jpg?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    if (jpg != null) {
      return jpg(this);
    }
    return orElse();
  }
}

abstract class Image_Jpg implements Image {
  const factory Image_Jpg({required final String base64}) = _$Image_JpgImpl;

  String get base64;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Image_JpgImplCopyWith<_$Image_JpgImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Image_AssetImplCopyWith<$Res> {
  factory _$$Image_AssetImplCopyWith(_$Image_AssetImpl value, $Res Function(_$Image_AssetImpl) then) =
      __$$Image_AssetImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String path});
}

/// @nodoc
class __$$Image_AssetImplCopyWithImpl<$Res> extends _$ImageCopyWithImpl<$Res, _$Image_AssetImpl>
    implements _$$Image_AssetImplCopyWith<$Res> {
  __$$Image_AssetImplCopyWithImpl(_$Image_AssetImpl _value, $Res Function(_$Image_AssetImpl) _then)
      : super(_value, _then);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? path = null,
  }) {
    return _then(_$Image_AssetImpl(
      path: null == path
          ? _value.path
          : path // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$Image_AssetImpl implements Image_Asset {
  const _$Image_AssetImpl({required this.path});

  @override
  final String path;

  @override
  String toString() {
    return 'Image.asset(path: $path)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Image_AssetImpl &&
            (identical(other.path, path) || other.path == path));
  }

  @override
  int get hashCode => Object.hash(runtimeType, path);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Image_AssetImplCopyWith<_$Image_AssetImpl> get copyWith =>
      __$$Image_AssetImplCopyWithImpl<_$Image_AssetImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) {
    return asset(path);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) {
    return asset?.call(path);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    if (asset != null) {
      return asset(path);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) {
    return asset(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    return asset?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    if (asset != null) {
      return asset(this);
    }
    return orElse();
  }
}

abstract class Image_Asset implements Image {
  const factory Image_Asset({required final String path}) = _$Image_AssetImpl;

  String get path;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Image_AssetImplCopyWith<_$Image_AssetImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$StartDisclosureResult {
  Organization get relyingParty => throw _privateConstructorUsedError;
  bool get sharedDataWithRelyingPartyBefore => throw _privateConstructorUsedError;
  DisclosureSessionType get sessionType => throw _privateConstructorUsedError;
  List<LocalizedString> get requestPurpose => throw _privateConstructorUsedError;
  String get requestOriginBaseUrl => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)
        request,
    required TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)
        requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult? Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(StartDisclosureResult_Request value) request,
    required TResult Function(StartDisclosureResult_RequestAttributesMissing value) requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StartDisclosureResult_Request value)? request,
    TResult? Function(StartDisclosureResult_RequestAttributesMissing value)? requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StartDisclosureResult_Request value)? request,
    TResult Function(StartDisclosureResult_RequestAttributesMissing value)? requestAttributesMissing,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $StartDisclosureResultCopyWith<StartDisclosureResult> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $StartDisclosureResultCopyWith<$Res> {
  factory $StartDisclosureResultCopyWith(StartDisclosureResult value, $Res Function(StartDisclosureResult) then) =
      _$StartDisclosureResultCopyWithImpl<$Res, StartDisclosureResult>;
  @useResult
  $Res call(
      {Organization relyingParty,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl});
}

/// @nodoc
class _$StartDisclosureResultCopyWithImpl<$Res, $Val extends StartDisclosureResult>
    implements $StartDisclosureResultCopyWith<$Res> {
  _$StartDisclosureResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
  }) {
    return _then(_value.copyWith(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _value.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _value.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _value.requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _value.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$StartDisclosureResult_RequestImplCopyWith<$Res> implements $StartDisclosureResultCopyWith<$Res> {
  factory _$$StartDisclosureResult_RequestImplCopyWith(
          _$StartDisclosureResult_RequestImpl value, $Res Function(_$StartDisclosureResult_RequestImpl) then) =
      __$$StartDisclosureResult_RequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {Organization relyingParty,
      RequestPolicy policy,
      List<DisclosureCard> requestedCards,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl,
      DisclosureType requestType});
}

/// @nodoc
class __$$StartDisclosureResult_RequestImplCopyWithImpl<$Res>
    extends _$StartDisclosureResultCopyWithImpl<$Res, _$StartDisclosureResult_RequestImpl>
    implements _$$StartDisclosureResult_RequestImplCopyWith<$Res> {
  __$$StartDisclosureResult_RequestImplCopyWithImpl(
      _$StartDisclosureResult_RequestImpl _value, $Res Function(_$StartDisclosureResult_RequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? policy = null,
    Object? requestedCards = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
    Object? requestType = null,
  }) {
    return _then(_$StartDisclosureResult_RequestImpl(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      policy: null == policy
          ? _value.policy
          : policy // ignore: cast_nullable_to_non_nullable
              as RequestPolicy,
      requestedCards: null == requestedCards
          ? _value._requestedCards
          : requestedCards // ignore: cast_nullable_to_non_nullable
              as List<DisclosureCard>,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _value.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _value.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _value._requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _value.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
      requestType: null == requestType
          ? _value.requestType
          : requestType // ignore: cast_nullable_to_non_nullable
              as DisclosureType,
    ));
  }
}

/// @nodoc

class _$StartDisclosureResult_RequestImpl implements StartDisclosureResult_Request {
  const _$StartDisclosureResult_RequestImpl(
      {required this.relyingParty,
      required this.policy,
      required final List<DisclosureCard> requestedCards,
      required this.sharedDataWithRelyingPartyBefore,
      required this.sessionType,
      required final List<LocalizedString> requestPurpose,
      required this.requestOriginBaseUrl,
      required this.requestType})
      : _requestedCards = requestedCards,
        _requestPurpose = requestPurpose;

  @override
  final Organization relyingParty;
  @override
  final RequestPolicy policy;
  final List<DisclosureCard> _requestedCards;
  @override
  List<DisclosureCard> get requestedCards {
    if (_requestedCards is EqualUnmodifiableListView) return _requestedCards;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestedCards);
  }

  @override
  final bool sharedDataWithRelyingPartyBefore;
  @override
  final DisclosureSessionType sessionType;
  final List<LocalizedString> _requestPurpose;
  @override
  List<LocalizedString> get requestPurpose {
    if (_requestPurpose is EqualUnmodifiableListView) return _requestPurpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestPurpose);
  }

  @override
  final String requestOriginBaseUrl;
  @override
  final DisclosureType requestType;

  @override
  String toString() {
    return 'StartDisclosureResult.request(relyingParty: $relyingParty, policy: $policy, requestedCards: $requestedCards, sharedDataWithRelyingPartyBefore: $sharedDataWithRelyingPartyBefore, sessionType: $sessionType, requestPurpose: $requestPurpose, requestOriginBaseUrl: $requestOriginBaseUrl, requestType: $requestType)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StartDisclosureResult_RequestImpl &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            (identical(other.policy, policy) || other.policy == policy) &&
            const DeepCollectionEquality().equals(other._requestedCards, _requestedCards) &&
            (identical(other.sharedDataWithRelyingPartyBefore, sharedDataWithRelyingPartyBefore) ||
                other.sharedDataWithRelyingPartyBefore == sharedDataWithRelyingPartyBefore) &&
            (identical(other.sessionType, sessionType) || other.sessionType == sessionType) &&
            const DeepCollectionEquality().equals(other._requestPurpose, _requestPurpose) &&
            (identical(other.requestOriginBaseUrl, requestOriginBaseUrl) ||
                other.requestOriginBaseUrl == requestOriginBaseUrl) &&
            (identical(other.requestType, requestType) || other.requestType == requestType));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      relyingParty,
      policy,
      const DeepCollectionEquality().hash(_requestedCards),
      sharedDataWithRelyingPartyBefore,
      sessionType,
      const DeepCollectionEquality().hash(_requestPurpose),
      requestOriginBaseUrl,
      requestType);

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$StartDisclosureResult_RequestImplCopyWith<_$StartDisclosureResult_RequestImpl> get copyWith =>
      __$$StartDisclosureResult_RequestImplCopyWithImpl<_$StartDisclosureResult_RequestImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)
        request,
    required TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)
        requestAttributesMissing,
  }) {
    return request(relyingParty, policy, requestedCards, sharedDataWithRelyingPartyBefore, sessionType, requestPurpose,
        requestOriginBaseUrl, requestType);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult? Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
  }) {
    return request?.call(relyingParty, policy, requestedCards, sharedDataWithRelyingPartyBefore, sessionType,
        requestPurpose, requestOriginBaseUrl, requestType);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (request != null) {
      return request(relyingParty, policy, requestedCards, sharedDataWithRelyingPartyBefore, sessionType,
          requestPurpose, requestOriginBaseUrl, requestType);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(StartDisclosureResult_Request value) request,
    required TResult Function(StartDisclosureResult_RequestAttributesMissing value) requestAttributesMissing,
  }) {
    return request(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StartDisclosureResult_Request value)? request,
    TResult? Function(StartDisclosureResult_RequestAttributesMissing value)? requestAttributesMissing,
  }) {
    return request?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StartDisclosureResult_Request value)? request,
    TResult Function(StartDisclosureResult_RequestAttributesMissing value)? requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (request != null) {
      return request(this);
    }
    return orElse();
  }
}

abstract class StartDisclosureResult_Request implements StartDisclosureResult {
  const factory StartDisclosureResult_Request(
      {required final Organization relyingParty,
      required final RequestPolicy policy,
      required final List<DisclosureCard> requestedCards,
      required final bool sharedDataWithRelyingPartyBefore,
      required final DisclosureSessionType sessionType,
      required final List<LocalizedString> requestPurpose,
      required final String requestOriginBaseUrl,
      required final DisclosureType requestType}) = _$StartDisclosureResult_RequestImpl;

  @override
  Organization get relyingParty;
  RequestPolicy get policy;
  List<DisclosureCard> get requestedCards;
  @override
  bool get sharedDataWithRelyingPartyBefore;
  @override
  DisclosureSessionType get sessionType;
  @override
  List<LocalizedString> get requestPurpose;
  @override
  String get requestOriginBaseUrl;
  DisclosureType get requestType;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$StartDisclosureResult_RequestImplCopyWith<_$StartDisclosureResult_RequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<$Res>
    implements $StartDisclosureResultCopyWith<$Res> {
  factory _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith(
          _$StartDisclosureResult_RequestAttributesMissingImpl value,
          $Res Function(_$StartDisclosureResult_RequestAttributesMissingImpl) then) =
      __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {Organization relyingParty,
      List<MissingAttribute> missingAttributes,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl});
}

/// @nodoc
class __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl<$Res>
    extends _$StartDisclosureResultCopyWithImpl<$Res, _$StartDisclosureResult_RequestAttributesMissingImpl>
    implements _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<$Res> {
  __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl(
      _$StartDisclosureResult_RequestAttributesMissingImpl _value,
      $Res Function(_$StartDisclosureResult_RequestAttributesMissingImpl) _then)
      : super(_value, _then);

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? missingAttributes = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
  }) {
    return _then(_$StartDisclosureResult_RequestAttributesMissingImpl(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      missingAttributes: null == missingAttributes
          ? _value._missingAttributes
          : missingAttributes // ignore: cast_nullable_to_non_nullable
              as List<MissingAttribute>,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _value.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _value.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _value._requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _value.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$StartDisclosureResult_RequestAttributesMissingImpl implements StartDisclosureResult_RequestAttributesMissing {
  const _$StartDisclosureResult_RequestAttributesMissingImpl(
      {required this.relyingParty,
      required final List<MissingAttribute> missingAttributes,
      required this.sharedDataWithRelyingPartyBefore,
      required this.sessionType,
      required final List<LocalizedString> requestPurpose,
      required this.requestOriginBaseUrl})
      : _missingAttributes = missingAttributes,
        _requestPurpose = requestPurpose;

  @override
  final Organization relyingParty;
  final List<MissingAttribute> _missingAttributes;
  @override
  List<MissingAttribute> get missingAttributes {
    if (_missingAttributes is EqualUnmodifiableListView) return _missingAttributes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_missingAttributes);
  }

  @override
  final bool sharedDataWithRelyingPartyBefore;
  @override
  final DisclosureSessionType sessionType;
  final List<LocalizedString> _requestPurpose;
  @override
  List<LocalizedString> get requestPurpose {
    if (_requestPurpose is EqualUnmodifiableListView) return _requestPurpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestPurpose);
  }

  @override
  final String requestOriginBaseUrl;

  @override
  String toString() {
    return 'StartDisclosureResult.requestAttributesMissing(relyingParty: $relyingParty, missingAttributes: $missingAttributes, sharedDataWithRelyingPartyBefore: $sharedDataWithRelyingPartyBefore, sessionType: $sessionType, requestPurpose: $requestPurpose, requestOriginBaseUrl: $requestOriginBaseUrl)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StartDisclosureResult_RequestAttributesMissingImpl &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._missingAttributes, _missingAttributes) &&
            (identical(other.sharedDataWithRelyingPartyBefore, sharedDataWithRelyingPartyBefore) ||
                other.sharedDataWithRelyingPartyBefore == sharedDataWithRelyingPartyBefore) &&
            (identical(other.sessionType, sessionType) || other.sessionType == sessionType) &&
            const DeepCollectionEquality().equals(other._requestPurpose, _requestPurpose) &&
            (identical(other.requestOriginBaseUrl, requestOriginBaseUrl) ||
                other.requestOriginBaseUrl == requestOriginBaseUrl));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      relyingParty,
      const DeepCollectionEquality().hash(_missingAttributes),
      sharedDataWithRelyingPartyBefore,
      sessionType,
      const DeepCollectionEquality().hash(_requestPurpose),
      requestOriginBaseUrl);

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<_$StartDisclosureResult_RequestAttributesMissingImpl>
      get copyWith => __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl<
          _$StartDisclosureResult_RequestAttributesMissingImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)
        request,
    required TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)
        requestAttributesMissing,
  }) {
    return requestAttributesMissing(relyingParty, missingAttributes, sharedDataWithRelyingPartyBefore, sessionType,
        requestPurpose, requestOriginBaseUrl);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult? Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
  }) {
    return requestAttributesMissing?.call(relyingParty, missingAttributes, sharedDataWithRelyingPartyBefore,
        sessionType, requestPurpose, requestOriginBaseUrl);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<DisclosureCard> requestedCards,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (requestAttributesMissing != null) {
      return requestAttributesMissing(relyingParty, missingAttributes, sharedDataWithRelyingPartyBefore, sessionType,
          requestPurpose, requestOriginBaseUrl);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(StartDisclosureResult_Request value) request,
    required TResult Function(StartDisclosureResult_RequestAttributesMissing value) requestAttributesMissing,
  }) {
    return requestAttributesMissing(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StartDisclosureResult_Request value)? request,
    TResult? Function(StartDisclosureResult_RequestAttributesMissing value)? requestAttributesMissing,
  }) {
    return requestAttributesMissing?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StartDisclosureResult_Request value)? request,
    TResult Function(StartDisclosureResult_RequestAttributesMissing value)? requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (requestAttributesMissing != null) {
      return requestAttributesMissing(this);
    }
    return orElse();
  }
}

abstract class StartDisclosureResult_RequestAttributesMissing implements StartDisclosureResult {
  const factory StartDisclosureResult_RequestAttributesMissing(
      {required final Organization relyingParty,
      required final List<MissingAttribute> missingAttributes,
      required final bool sharedDataWithRelyingPartyBefore,
      required final DisclosureSessionType sessionType,
      required final List<LocalizedString> requestPurpose,
      required final String requestOriginBaseUrl}) = _$StartDisclosureResult_RequestAttributesMissingImpl;

  @override
  Organization get relyingParty;
  List<MissingAttribute> get missingAttributes;
  @override
  bool get sharedDataWithRelyingPartyBefore;
  @override
  DisclosureSessionType get sessionType;
  @override
  List<LocalizedString> get requestPurpose;
  @override
  String get requestOriginBaseUrl;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<_$StartDisclosureResult_RequestAttributesMissingImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$WalletEvent {
  String get dateTime => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)
        disclosure,
    required TResult Function(String dateTime, Card card) issuance,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)?
        disclosure,
    TResult? Function(String dateTime, Card card)? issuance,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)?
        disclosure,
    TResult Function(String dateTime, Card card)? issuance,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletEvent_Disclosure value) disclosure,
    required TResult Function(WalletEvent_Issuance value) issuance,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletEvent_Disclosure value)? disclosure,
    TResult? Function(WalletEvent_Issuance value)? issuance,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletEvent_Disclosure value)? disclosure,
    TResult Function(WalletEvent_Issuance value)? issuance,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $WalletEventCopyWith<WalletEvent> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $WalletEventCopyWith<$Res> {
  factory $WalletEventCopyWith(WalletEvent value, $Res Function(WalletEvent) then) =
      _$WalletEventCopyWithImpl<$Res, WalletEvent>;
  @useResult
  $Res call({String dateTime});
}

/// @nodoc
class _$WalletEventCopyWithImpl<$Res, $Val extends WalletEvent> implements $WalletEventCopyWith<$Res> {
  _$WalletEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? dateTime = null,
  }) {
    return _then(_value.copyWith(
      dateTime: null == dateTime
          ? _value.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$WalletEvent_DisclosureImplCopyWith<$Res> implements $WalletEventCopyWith<$Res> {
  factory _$$WalletEvent_DisclosureImplCopyWith(
          _$WalletEvent_DisclosureImpl value, $Res Function(_$WalletEvent_DisclosureImpl) then) =
      __$$WalletEvent_DisclosureImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String dateTime,
      Organization relyingParty,
      List<LocalizedString> purpose,
      List<DisclosureCard>? requestedCards,
      RequestPolicy requestPolicy,
      DisclosureStatus status,
      DisclosureType type});
}

/// @nodoc
class __$$WalletEvent_DisclosureImplCopyWithImpl<$Res>
    extends _$WalletEventCopyWithImpl<$Res, _$WalletEvent_DisclosureImpl>
    implements _$$WalletEvent_DisclosureImplCopyWith<$Res> {
  __$$WalletEvent_DisclosureImplCopyWithImpl(
      _$WalletEvent_DisclosureImpl _value, $Res Function(_$WalletEvent_DisclosureImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? dateTime = null,
    Object? relyingParty = null,
    Object? purpose = null,
    Object? requestedCards = freezed,
    Object? requestPolicy = null,
    Object? status = null,
    Object? type = null,
  }) {
    return _then(_$WalletEvent_DisclosureImpl(
      dateTime: null == dateTime
          ? _value.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      purpose: null == purpose
          ? _value._purpose
          : purpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestedCards: freezed == requestedCards
          ? _value._requestedCards
          : requestedCards // ignore: cast_nullable_to_non_nullable
              as List<DisclosureCard>?,
      requestPolicy: null == requestPolicy
          ? _value.requestPolicy
          : requestPolicy // ignore: cast_nullable_to_non_nullable
              as RequestPolicy,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as DisclosureStatus,
      type: null == type
          ? _value.type
          : type // ignore: cast_nullable_to_non_nullable
              as DisclosureType,
    ));
  }
}

/// @nodoc

class _$WalletEvent_DisclosureImpl implements WalletEvent_Disclosure {
  const _$WalletEvent_DisclosureImpl(
      {required this.dateTime,
      required this.relyingParty,
      required final List<LocalizedString> purpose,
      final List<DisclosureCard>? requestedCards,
      required this.requestPolicy,
      required this.status,
      required this.type})
      : _purpose = purpose,
        _requestedCards = requestedCards;

  @override
  final String dateTime;
  @override
  final Organization relyingParty;
  final List<LocalizedString> _purpose;
  @override
  List<LocalizedString> get purpose {
    if (_purpose is EqualUnmodifiableListView) return _purpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_purpose);
  }

  final List<DisclosureCard>? _requestedCards;
  @override
  List<DisclosureCard>? get requestedCards {
    final value = _requestedCards;
    if (value == null) return null;
    if (_requestedCards is EqualUnmodifiableListView) return _requestedCards;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  @override
  final RequestPolicy requestPolicy;
  @override
  final DisclosureStatus status;
  @override
  final DisclosureType type;

  @override
  String toString() {
    return 'WalletEvent.disclosure(dateTime: $dateTime, relyingParty: $relyingParty, purpose: $purpose, requestedCards: $requestedCards, requestPolicy: $requestPolicy, status: $status, type: $type)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletEvent_DisclosureImpl &&
            (identical(other.dateTime, dateTime) || other.dateTime == dateTime) &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._purpose, _purpose) &&
            const DeepCollectionEquality().equals(other._requestedCards, _requestedCards) &&
            (identical(other.requestPolicy, requestPolicy) || other.requestPolicy == requestPolicy) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.type, type) || other.type == type));
  }

  @override
  int get hashCode => Object.hash(runtimeType, dateTime, relyingParty, const DeepCollectionEquality().hash(_purpose),
      const DeepCollectionEquality().hash(_requestedCards), requestPolicy, status, type);

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletEvent_DisclosureImplCopyWith<_$WalletEvent_DisclosureImpl> get copyWith =>
      __$$WalletEvent_DisclosureImplCopyWithImpl<_$WalletEvent_DisclosureImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)
        disclosure,
    required TResult Function(String dateTime, Card card) issuance,
  }) {
    return disclosure(dateTime, relyingParty, purpose, requestedCards, requestPolicy, status, type);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)?
        disclosure,
    TResult? Function(String dateTime, Card card)? issuance,
  }) {
    return disclosure?.call(dateTime, relyingParty, purpose, requestedCards, requestPolicy, status, type);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)?
        disclosure,
    TResult Function(String dateTime, Card card)? issuance,
    required TResult orElse(),
  }) {
    if (disclosure != null) {
      return disclosure(dateTime, relyingParty, purpose, requestedCards, requestPolicy, status, type);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletEvent_Disclosure value) disclosure,
    required TResult Function(WalletEvent_Issuance value) issuance,
  }) {
    return disclosure(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletEvent_Disclosure value)? disclosure,
    TResult? Function(WalletEvent_Issuance value)? issuance,
  }) {
    return disclosure?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletEvent_Disclosure value)? disclosure,
    TResult Function(WalletEvent_Issuance value)? issuance,
    required TResult orElse(),
  }) {
    if (disclosure != null) {
      return disclosure(this);
    }
    return orElse();
  }
}

abstract class WalletEvent_Disclosure implements WalletEvent {
  const factory WalletEvent_Disclosure(
      {required final String dateTime,
      required final Organization relyingParty,
      required final List<LocalizedString> purpose,
      final List<DisclosureCard>? requestedCards,
      required final RequestPolicy requestPolicy,
      required final DisclosureStatus status,
      required final DisclosureType type}) = _$WalletEvent_DisclosureImpl;

  @override
  String get dateTime;
  Organization get relyingParty;
  List<LocalizedString> get purpose;
  List<DisclosureCard>? get requestedCards;
  RequestPolicy get requestPolicy;
  DisclosureStatus get status;
  DisclosureType get type;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WalletEvent_DisclosureImplCopyWith<_$WalletEvent_DisclosureImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletEvent_IssuanceImplCopyWith<$Res> implements $WalletEventCopyWith<$Res> {
  factory _$$WalletEvent_IssuanceImplCopyWith(
          _$WalletEvent_IssuanceImpl value, $Res Function(_$WalletEvent_IssuanceImpl) then) =
      __$$WalletEvent_IssuanceImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String dateTime, Card card});
}

/// @nodoc
class __$$WalletEvent_IssuanceImplCopyWithImpl<$Res> extends _$WalletEventCopyWithImpl<$Res, _$WalletEvent_IssuanceImpl>
    implements _$$WalletEvent_IssuanceImplCopyWith<$Res> {
  __$$WalletEvent_IssuanceImplCopyWithImpl(
      _$WalletEvent_IssuanceImpl _value, $Res Function(_$WalletEvent_IssuanceImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? dateTime = null,
    Object? card = null,
  }) {
    return _then(_$WalletEvent_IssuanceImpl(
      dateTime: null == dateTime
          ? _value.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
      card: null == card
          ? _value.card
          : card // ignore: cast_nullable_to_non_nullable
              as Card,
    ));
  }
}

/// @nodoc

class _$WalletEvent_IssuanceImpl implements WalletEvent_Issuance {
  const _$WalletEvent_IssuanceImpl({required this.dateTime, required this.card});

  @override
  final String dateTime;
  @override
  final Card card;

  @override
  String toString() {
    return 'WalletEvent.issuance(dateTime: $dateTime, card: $card)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletEvent_IssuanceImpl &&
            (identical(other.dateTime, dateTime) || other.dateTime == dateTime) &&
            (identical(other.card, card) || other.card == card));
  }

  @override
  int get hashCode => Object.hash(runtimeType, dateTime, card);

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletEvent_IssuanceImplCopyWith<_$WalletEvent_IssuanceImpl> get copyWith =>
      __$$WalletEvent_IssuanceImplCopyWithImpl<_$WalletEvent_IssuanceImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)
        disclosure,
    required TResult Function(String dateTime, Card card) issuance,
  }) {
    return issuance(dateTime, card);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)?
        disclosure,
    TResult? Function(String dateTime, Card card)? issuance,
  }) {
    return issuance?.call(dateTime, card);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType type)?
        disclosure,
    TResult Function(String dateTime, Card card)? issuance,
    required TResult orElse(),
  }) {
    if (issuance != null) {
      return issuance(dateTime, card);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletEvent_Disclosure value) disclosure,
    required TResult Function(WalletEvent_Issuance value) issuance,
  }) {
    return issuance(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletEvent_Disclosure value)? disclosure,
    TResult? Function(WalletEvent_Issuance value)? issuance,
  }) {
    return issuance?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletEvent_Disclosure value)? disclosure,
    TResult Function(WalletEvent_Issuance value)? issuance,
    required TResult orElse(),
  }) {
    if (issuance != null) {
      return issuance(this);
    }
    return orElse();
  }
}

abstract class WalletEvent_Issuance implements WalletEvent {
  const factory WalletEvent_Issuance({required final String dateTime, required final Card card}) =
      _$WalletEvent_IssuanceImpl;

  @override
  String get dateTime;
  Card get card;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WalletEvent_IssuanceImplCopyWith<_$WalletEvent_IssuanceImpl> get copyWith => throw _privateConstructorUsedError;
}

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

class _$WalletInstructionError_IncorrectPinImpl implements WalletInstructionError_IncorrectPin {
  const _$WalletInstructionError_IncorrectPinImpl({required this.attemptsLeftInRound, required this.isFinalRound});

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

abstract class WalletInstructionError_IncorrectPin implements WalletInstructionError {
  const factory WalletInstructionError_IncorrectPin(
      {required final int attemptsLeftInRound,
      required final bool isFinalRound}) = _$WalletInstructionError_IncorrectPinImpl;

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

class _$WalletInstructionError_TimeoutImpl implements WalletInstructionError_Timeout {
  const _$WalletInstructionError_TimeoutImpl({required this.timeoutMillis});

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

abstract class WalletInstructionError_Timeout implements WalletInstructionError {
  const factory WalletInstructionError_Timeout({required final int timeoutMillis}) =
      _$WalletInstructionError_TimeoutImpl;

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

class _$WalletInstructionError_BlockedImpl implements WalletInstructionError_Blocked {
  const _$WalletInstructionError_BlockedImpl();

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

abstract class WalletInstructionError_Blocked implements WalletInstructionError {
  const factory WalletInstructionError_Blocked() = _$WalletInstructionError_BlockedImpl;
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

class _$WalletInstructionResult_OkImpl implements WalletInstructionResult_Ok {
  const _$WalletInstructionResult_OkImpl();

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

abstract class WalletInstructionResult_Ok implements WalletInstructionResult {
  const factory WalletInstructionResult_Ok() = _$WalletInstructionResult_OkImpl;
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

class _$WalletInstructionResult_InstructionErrorImpl implements WalletInstructionResult_InstructionError {
  const _$WalletInstructionResult_InstructionErrorImpl({required this.error});

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

abstract class WalletInstructionResult_InstructionError implements WalletInstructionResult {
  const factory WalletInstructionResult_InstructionError({required final WalletInstructionError error}) =
      _$WalletInstructionResult_InstructionErrorImpl;

  WalletInstructionError get error;

  /// Create a copy of WalletInstructionResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WalletInstructionResult_InstructionErrorImplCopyWith<_$WalletInstructionResult_InstructionErrorImpl>
      get copyWith => throw _privateConstructorUsedError;
}
