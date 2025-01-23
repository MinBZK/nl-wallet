// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'card.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

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

class _$CardPersistence_InMemoryImpl extends CardPersistence_InMemory {
  const _$CardPersistence_InMemoryImpl() : super._();

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

abstract class CardPersistence_InMemory extends CardPersistence {
  const factory CardPersistence_InMemory() = _$CardPersistence_InMemoryImpl;
  const CardPersistence_InMemory._() : super._();
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

class _$CardPersistence_StoredImpl extends CardPersistence_Stored {
  const _$CardPersistence_StoredImpl({required this.id}) : super._();

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

abstract class CardPersistence_Stored extends CardPersistence {
  const factory CardPersistence_Stored({required final String id}) = _$CardPersistence_StoredImpl;
  const CardPersistence_Stored._() : super._();

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

class _$CardValue_StringImpl extends CardValue_String {
  const _$CardValue_StringImpl({required this.value}) : super._();

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

abstract class CardValue_String extends CardValue {
  const factory CardValue_String({required final String value}) = _$CardValue_StringImpl;
  const CardValue_String._() : super._();

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

class _$CardValue_BooleanImpl extends CardValue_Boolean {
  const _$CardValue_BooleanImpl({required this.value}) : super._();

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

abstract class CardValue_Boolean extends CardValue {
  const factory CardValue_Boolean({required final bool value}) = _$CardValue_BooleanImpl;
  const CardValue_Boolean._() : super._();

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

class _$CardValue_DateImpl extends CardValue_Date {
  const _$CardValue_DateImpl({required this.value}) : super._();

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

abstract class CardValue_Date extends CardValue {
  const factory CardValue_Date({required final String value}) = _$CardValue_DateImpl;
  const CardValue_Date._() : super._();

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

class _$CardValue_GenderImpl extends CardValue_Gender {
  const _$CardValue_GenderImpl({required this.value}) : super._();

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

abstract class CardValue_Gender extends CardValue {
  const factory CardValue_Gender({required final GenderCardValue value}) = _$CardValue_GenderImpl;
  const CardValue_Gender._() : super._();

  @override
  GenderCardValue get value;

  /// Create a copy of CardValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CardValue_GenderImplCopyWith<_$CardValue_GenderImpl> get copyWith => throw _privateConstructorUsedError;
}
