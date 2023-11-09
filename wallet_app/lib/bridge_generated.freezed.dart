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
}

/// @nodoc

class _$CardPersistence_InMemoryImpl implements CardPersistence_InMemory {
  const _$CardPersistence_InMemoryImpl();

  @override
  String toString() {
    return 'CardPersistence.inMemory()';
  }

  @override
  bool operator ==(dynamic other) {
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
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardPersistence_StoredImpl &&
            (identical(other.id, id) || other.id == id));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id);

  @JsonKey(ignore: true)
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
  @JsonKey(ignore: true)
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
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardValue_StringImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @JsonKey(ignore: true)
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
  @JsonKey(ignore: true)
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
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardValue_BooleanImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @JsonKey(ignore: true)
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
  @JsonKey(ignore: true)
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
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardValue_DateImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @JsonKey(ignore: true)
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
  @JsonKey(ignore: true)
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
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CardValue_GenderImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @JsonKey(ignore: true)
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
  @JsonKey(ignore: true)
  _$$CardValue_GenderImplCopyWith<_$CardValue_GenderImpl> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$StartDisclosureResult {
  RelyingParty get relyingParty => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards) request,
    required TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)
        requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult? Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
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

  @JsonKey(ignore: true)
  $StartDisclosureResultCopyWith<StartDisclosureResult> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $StartDisclosureResultCopyWith<$Res> {
  factory $StartDisclosureResultCopyWith(StartDisclosureResult value, $Res Function(StartDisclosureResult) then) =
      _$StartDisclosureResultCopyWithImpl<$Res, StartDisclosureResult>;
  @useResult
  $Res call({RelyingParty relyingParty});
}

/// @nodoc
class _$StartDisclosureResultCopyWithImpl<$Res, $Val extends StartDisclosureResult>
    implements $StartDisclosureResultCopyWith<$Res> {
  _$StartDisclosureResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
  }) {
    return _then(_value.copyWith(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as RelyingParty,
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
  $Res call({RelyingParty relyingParty, List<RequestedCard> requestedCards});
}

/// @nodoc
class __$$StartDisclosureResult_RequestImplCopyWithImpl<$Res>
    extends _$StartDisclosureResultCopyWithImpl<$Res, _$StartDisclosureResult_RequestImpl>
    implements _$$StartDisclosureResult_RequestImplCopyWith<$Res> {
  __$$StartDisclosureResult_RequestImplCopyWithImpl(
      _$StartDisclosureResult_RequestImpl _value, $Res Function(_$StartDisclosureResult_RequestImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? requestedCards = null,
  }) {
    return _then(_$StartDisclosureResult_RequestImpl(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as RelyingParty,
      requestedCards: null == requestedCards
          ? _value._requestedCards
          : requestedCards // ignore: cast_nullable_to_non_nullable
              as List<RequestedCard>,
    ));
  }
}

/// @nodoc

class _$StartDisclosureResult_RequestImpl implements StartDisclosureResult_Request {
  const _$StartDisclosureResult_RequestImpl(
      {required this.relyingParty, required final List<RequestedCard> requestedCards})
      : _requestedCards = requestedCards;

  @override
  final RelyingParty relyingParty;
  final List<RequestedCard> _requestedCards;
  @override
  List<RequestedCard> get requestedCards {
    if (_requestedCards is EqualUnmodifiableListView) return _requestedCards;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestedCards);
  }

  @override
  String toString() {
    return 'StartDisclosureResult.request(relyingParty: $relyingParty, requestedCards: $requestedCards)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StartDisclosureResult_RequestImpl &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._requestedCards, _requestedCards));
  }

  @override
  int get hashCode => Object.hash(runtimeType, relyingParty, const DeepCollectionEquality().hash(_requestedCards));

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$StartDisclosureResult_RequestImplCopyWith<_$StartDisclosureResult_RequestImpl> get copyWith =>
      __$$StartDisclosureResult_RequestImplCopyWithImpl<_$StartDisclosureResult_RequestImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards) request,
    required TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)
        requestAttributesMissing,
  }) {
    return request(relyingParty, requestedCards);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult? Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
  }) {
    return request?.call(relyingParty, requestedCards);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (request != null) {
      return request(relyingParty, requestedCards);
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
      {required final RelyingParty relyingParty,
      required final List<RequestedCard> requestedCards}) = _$StartDisclosureResult_RequestImpl;

  @override
  RelyingParty get relyingParty;
  List<RequestedCard> get requestedCards;
  @override
  @JsonKey(ignore: true)
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
  $Res call({RelyingParty relyingParty, List<MissingAttribute> missingAttributes});
}

/// @nodoc
class __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl<$Res>
    extends _$StartDisclosureResultCopyWithImpl<$Res, _$StartDisclosureResult_RequestAttributesMissingImpl>
    implements _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<$Res> {
  __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl(
      _$StartDisclosureResult_RequestAttributesMissingImpl _value,
      $Res Function(_$StartDisclosureResult_RequestAttributesMissingImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? missingAttributes = null,
  }) {
    return _then(_$StartDisclosureResult_RequestAttributesMissingImpl(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as RelyingParty,
      missingAttributes: null == missingAttributes
          ? _value._missingAttributes
          : missingAttributes // ignore: cast_nullable_to_non_nullable
              as List<MissingAttribute>,
    ));
  }
}

/// @nodoc

class _$StartDisclosureResult_RequestAttributesMissingImpl implements StartDisclosureResult_RequestAttributesMissing {
  const _$StartDisclosureResult_RequestAttributesMissingImpl(
      {required this.relyingParty, required final List<MissingAttribute> missingAttributes})
      : _missingAttributes = missingAttributes;

  @override
  final RelyingParty relyingParty;
  final List<MissingAttribute> _missingAttributes;
  @override
  List<MissingAttribute> get missingAttributes {
    if (_missingAttributes is EqualUnmodifiableListView) return _missingAttributes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_missingAttributes);
  }

  @override
  String toString() {
    return 'StartDisclosureResult.requestAttributesMissing(relyingParty: $relyingParty, missingAttributes: $missingAttributes)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StartDisclosureResult_RequestAttributesMissingImpl &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._missingAttributes, _missingAttributes));
  }

  @override
  int get hashCode => Object.hash(runtimeType, relyingParty, const DeepCollectionEquality().hash(_missingAttributes));

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<_$StartDisclosureResult_RequestAttributesMissingImpl>
      get copyWith => __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl<
          _$StartDisclosureResult_RequestAttributesMissingImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards) request,
    required TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)
        requestAttributesMissing,
  }) {
    return requestAttributesMissing(relyingParty, missingAttributes);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult? Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
  }) {
    return requestAttributesMissing?.call(relyingParty, missingAttributes);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (requestAttributesMissing != null) {
      return requestAttributesMissing(relyingParty, missingAttributes);
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
      {required final RelyingParty relyingParty,
      required final List<MissingAttribute> missingAttributes}) = _$StartDisclosureResult_RequestAttributesMissingImpl;

  @override
  RelyingParty get relyingParty;
  List<MissingAttribute> get missingAttributes;
  @override
  @JsonKey(ignore: true)
  _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<_$StartDisclosureResult_RequestAttributesMissingImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$WalletInstructionResult {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? ok,
    TResult? Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult? Function(int timeoutMillis)? timeout,
    TResult? Function()? blocked,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? ok,
    TResult Function(int leftoverAttempts, bool isFinalAttempt)? incorrectPin,
    TResult Function(int timeoutMillis)? timeout,
    TResult Function()? blocked,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletInstructionResult_Ok value) ok,
    required TResult Function(WalletInstructionResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionResult_Timeout value) timeout,
    required TResult Function(WalletInstructionResult_Blocked value) blocked,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionResult_Ok value)? ok,
    TResult? Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionResult_Timeout value)? timeout,
    TResult? Function(WalletInstructionResult_Blocked value)? blocked,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionResult_Ok value)? ok,
    TResult Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionResult_Timeout value)? timeout,
    TResult Function(WalletInstructionResult_Blocked value)? blocked,
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
}

/// @nodoc

class _$WalletInstructionResult_OkImpl implements WalletInstructionResult_Ok {
  const _$WalletInstructionResult_OkImpl();

  @override
  String toString() {
    return 'WalletInstructionResult.ok()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$WalletInstructionResult_OkImpl);
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
    required TResult Function(WalletInstructionResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionResult_Timeout value) timeout,
    required TResult Function(WalletInstructionResult_Blocked value) blocked,
  }) {
    return ok(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionResult_Ok value)? ok,
    TResult? Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionResult_Timeout value)? timeout,
    TResult? Function(WalletInstructionResult_Blocked value)? blocked,
  }) {
    return ok?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionResult_Ok value)? ok,
    TResult Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionResult_Timeout value)? timeout,
    TResult Function(WalletInstructionResult_Blocked value)? blocked,
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
abstract class _$$WalletInstructionResult_IncorrectPinImplCopyWith<$Res> {
  factory _$$WalletInstructionResult_IncorrectPinImplCopyWith(_$WalletInstructionResult_IncorrectPinImpl value,
          $Res Function(_$WalletInstructionResult_IncorrectPinImpl) then) =
      __$$WalletInstructionResult_IncorrectPinImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int leftoverAttempts, bool isFinalAttempt});
}

/// @nodoc
class __$$WalletInstructionResult_IncorrectPinImplCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_IncorrectPinImpl>
    implements _$$WalletInstructionResult_IncorrectPinImplCopyWith<$Res> {
  __$$WalletInstructionResult_IncorrectPinImplCopyWithImpl(_$WalletInstructionResult_IncorrectPinImpl _value,
      $Res Function(_$WalletInstructionResult_IncorrectPinImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? leftoverAttempts = null,
    Object? isFinalAttempt = null,
  }) {
    return _then(_$WalletInstructionResult_IncorrectPinImpl(
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

class _$WalletInstructionResult_IncorrectPinImpl implements WalletInstructionResult_IncorrectPin {
  const _$WalletInstructionResult_IncorrectPinImpl({required this.leftoverAttempts, required this.isFinalAttempt});

  @override
  final int leftoverAttempts;
  @override
  final bool isFinalAttempt;

  @override
  String toString() {
    return 'WalletInstructionResult.incorrectPin(leftoverAttempts: $leftoverAttempts, isFinalAttempt: $isFinalAttempt)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletInstructionResult_IncorrectPinImpl &&
            (identical(other.leftoverAttempts, leftoverAttempts) || other.leftoverAttempts == leftoverAttempts) &&
            (identical(other.isFinalAttempt, isFinalAttempt) || other.isFinalAttempt == isFinalAttempt));
  }

  @override
  int get hashCode => Object.hash(runtimeType, leftoverAttempts, isFinalAttempt);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletInstructionResult_IncorrectPinImplCopyWith<_$WalletInstructionResult_IncorrectPinImpl> get copyWith =>
      __$$WalletInstructionResult_IncorrectPinImplCopyWithImpl<_$WalletInstructionResult_IncorrectPinImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
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
    required TResult Function(WalletInstructionResult_Ok value) ok,
    required TResult Function(WalletInstructionResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionResult_Timeout value) timeout,
    required TResult Function(WalletInstructionResult_Blocked value) blocked,
  }) {
    return incorrectPin(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionResult_Ok value)? ok,
    TResult? Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionResult_Timeout value)? timeout,
    TResult? Function(WalletInstructionResult_Blocked value)? blocked,
  }) {
    return incorrectPin?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionResult_Ok value)? ok,
    TResult Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionResult_Timeout value)? timeout,
    TResult Function(WalletInstructionResult_Blocked value)? blocked,
    required TResult orElse(),
  }) {
    if (incorrectPin != null) {
      return incorrectPin(this);
    }
    return orElse();
  }
}

abstract class WalletInstructionResult_IncorrectPin implements WalletInstructionResult {
  const factory WalletInstructionResult_IncorrectPin(
      {required final int leftoverAttempts,
      required final bool isFinalAttempt}) = _$WalletInstructionResult_IncorrectPinImpl;

  int get leftoverAttempts;
  bool get isFinalAttempt;
  @JsonKey(ignore: true)
  _$$WalletInstructionResult_IncorrectPinImplCopyWith<_$WalletInstructionResult_IncorrectPinImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletInstructionResult_TimeoutImplCopyWith<$Res> {
  factory _$$WalletInstructionResult_TimeoutImplCopyWith(
          _$WalletInstructionResult_TimeoutImpl value, $Res Function(_$WalletInstructionResult_TimeoutImpl) then) =
      __$$WalletInstructionResult_TimeoutImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int timeoutMillis});
}

/// @nodoc
class __$$WalletInstructionResult_TimeoutImplCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_TimeoutImpl>
    implements _$$WalletInstructionResult_TimeoutImplCopyWith<$Res> {
  __$$WalletInstructionResult_TimeoutImplCopyWithImpl(
      _$WalletInstructionResult_TimeoutImpl _value, $Res Function(_$WalletInstructionResult_TimeoutImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timeoutMillis = null,
  }) {
    return _then(_$WalletInstructionResult_TimeoutImpl(
      timeoutMillis: null == timeoutMillis
          ? _value.timeoutMillis
          : timeoutMillis // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$WalletInstructionResult_TimeoutImpl implements WalletInstructionResult_Timeout {
  const _$WalletInstructionResult_TimeoutImpl({required this.timeoutMillis});

  @override
  final int timeoutMillis;

  @override
  String toString() {
    return 'WalletInstructionResult.timeout(timeoutMillis: $timeoutMillis)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletInstructionResult_TimeoutImpl &&
            (identical(other.timeoutMillis, timeoutMillis) || other.timeoutMillis == timeoutMillis));
  }

  @override
  int get hashCode => Object.hash(runtimeType, timeoutMillis);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletInstructionResult_TimeoutImplCopyWith<_$WalletInstructionResult_TimeoutImpl> get copyWith =>
      __$$WalletInstructionResult_TimeoutImplCopyWithImpl<_$WalletInstructionResult_TimeoutImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() ok,
    required TResult Function(int leftoverAttempts, bool isFinalAttempt) incorrectPin,
    required TResult Function(int timeoutMillis) timeout,
    required TResult Function() blocked,
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
    required TResult Function(WalletInstructionResult_Ok value) ok,
    required TResult Function(WalletInstructionResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionResult_Timeout value) timeout,
    required TResult Function(WalletInstructionResult_Blocked value) blocked,
  }) {
    return timeout(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionResult_Ok value)? ok,
    TResult? Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionResult_Timeout value)? timeout,
    TResult? Function(WalletInstructionResult_Blocked value)? blocked,
  }) {
    return timeout?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionResult_Ok value)? ok,
    TResult Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionResult_Timeout value)? timeout,
    TResult Function(WalletInstructionResult_Blocked value)? blocked,
    required TResult orElse(),
  }) {
    if (timeout != null) {
      return timeout(this);
    }
    return orElse();
  }
}

abstract class WalletInstructionResult_Timeout implements WalletInstructionResult {
  const factory WalletInstructionResult_Timeout({required final int timeoutMillis}) =
      _$WalletInstructionResult_TimeoutImpl;

  int get timeoutMillis;
  @JsonKey(ignore: true)
  _$$WalletInstructionResult_TimeoutImplCopyWith<_$WalletInstructionResult_TimeoutImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletInstructionResult_BlockedImplCopyWith<$Res> {
  factory _$$WalletInstructionResult_BlockedImplCopyWith(
          _$WalletInstructionResult_BlockedImpl value, $Res Function(_$WalletInstructionResult_BlockedImpl) then) =
      __$$WalletInstructionResult_BlockedImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$WalletInstructionResult_BlockedImplCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_BlockedImpl>
    implements _$$WalletInstructionResult_BlockedImplCopyWith<$Res> {
  __$$WalletInstructionResult_BlockedImplCopyWithImpl(
      _$WalletInstructionResult_BlockedImpl _value, $Res Function(_$WalletInstructionResult_BlockedImpl) _then)
      : super(_value, _then);
}

/// @nodoc

class _$WalletInstructionResult_BlockedImpl implements WalletInstructionResult_Blocked {
  const _$WalletInstructionResult_BlockedImpl();

  @override
  String toString() {
    return 'WalletInstructionResult.blocked()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is _$WalletInstructionResult_BlockedImpl);
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
    required TResult Function(WalletInstructionResult_Ok value) ok,
    required TResult Function(WalletInstructionResult_IncorrectPin value) incorrectPin,
    required TResult Function(WalletInstructionResult_Timeout value) timeout,
    required TResult Function(WalletInstructionResult_Blocked value) blocked,
  }) {
    return blocked(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletInstructionResult_Ok value)? ok,
    TResult? Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult? Function(WalletInstructionResult_Timeout value)? timeout,
    TResult? Function(WalletInstructionResult_Blocked value)? blocked,
  }) {
    return blocked?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletInstructionResult_Ok value)? ok,
    TResult Function(WalletInstructionResult_IncorrectPin value)? incorrectPin,
    TResult Function(WalletInstructionResult_Timeout value)? timeout,
    TResult Function(WalletInstructionResult_Blocked value)? blocked,
    required TResult orElse(),
  }) {
    if (blocked != null) {
      return blocked(this);
    }
    return orElse();
  }
}

abstract class WalletInstructionResult_Blocked implements WalletInstructionResult {
  const factory WalletInstructionResult_Blocked() = _$WalletInstructionResult_BlockedImpl;
}
