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
abstract class _$$CardPersistence_InMemoryCopyWith<$Res> {
  factory _$$CardPersistence_InMemoryCopyWith(
          _$CardPersistence_InMemory value, $Res Function(_$CardPersistence_InMemory) then) =
      __$$CardPersistence_InMemoryCopyWithImpl<$Res>;
}

/// @nodoc
class __$$CardPersistence_InMemoryCopyWithImpl<$Res>
    extends _$CardPersistenceCopyWithImpl<$Res, _$CardPersistence_InMemory>
    implements _$$CardPersistence_InMemoryCopyWith<$Res> {
  __$$CardPersistence_InMemoryCopyWithImpl(
      _$CardPersistence_InMemory _value, $Res Function(_$CardPersistence_InMemory) _then)
      : super(_value, _then);
}

/// @nodoc

class _$CardPersistence_InMemory implements CardPersistence_InMemory {
  const _$CardPersistence_InMemory();

  @override
  String toString() {
    return 'CardPersistence.inMemory()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$CardPersistence_InMemory);
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
  const factory CardPersistence_InMemory() = _$CardPersistence_InMemory;
}

/// @nodoc
abstract class _$$CardPersistence_StoredCopyWith<$Res> {
  factory _$$CardPersistence_StoredCopyWith(
          _$CardPersistence_Stored value, $Res Function(_$CardPersistence_Stored) then) =
      __$$CardPersistence_StoredCopyWithImpl<$Res>;
  @useResult
  $Res call({String id});
}

/// @nodoc
class __$$CardPersistence_StoredCopyWithImpl<$Res> extends _$CardPersistenceCopyWithImpl<$Res, _$CardPersistence_Stored>
    implements _$$CardPersistence_StoredCopyWith<$Res> {
  __$$CardPersistence_StoredCopyWithImpl(_$CardPersistence_Stored _value, $Res Function(_$CardPersistence_Stored) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
  }) {
    return _then(_$CardPersistence_Stored(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$CardPersistence_Stored implements CardPersistence_Stored {
  const _$CardPersistence_Stored({required this.id});

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
            other is _$CardPersistence_Stored &&
            (identical(other.id, id) || other.id == id));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$CardPersistence_StoredCopyWith<_$CardPersistence_Stored> get copyWith =>
      __$$CardPersistence_StoredCopyWithImpl<_$CardPersistence_Stored>(this, _$identity);

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
  const factory CardPersistence_Stored({required final String id}) = _$CardPersistence_Stored;

  String get id;
  @JsonKey(ignore: true)
  _$$CardPersistence_StoredCopyWith<_$CardPersistence_Stored> get copyWith => throw _privateConstructorUsedError;
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
abstract class _$$CardValue_StringCopyWith<$Res> {
  factory _$$CardValue_StringCopyWith(_$CardValue_String value, $Res Function(_$CardValue_String) then) =
      __$$CardValue_StringCopyWithImpl<$Res>;
  @useResult
  $Res call({String value});
}

/// @nodoc
class __$$CardValue_StringCopyWithImpl<$Res> extends _$CardValueCopyWithImpl<$Res, _$CardValue_String>
    implements _$$CardValue_StringCopyWith<$Res> {
  __$$CardValue_StringCopyWithImpl(_$CardValue_String _value, $Res Function(_$CardValue_String) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$CardValue_String(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$CardValue_String implements CardValue_String {
  const _$CardValue_String({required this.value});

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
            other is _$CardValue_String &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$CardValue_StringCopyWith<_$CardValue_String> get copyWith =>
      __$$CardValue_StringCopyWithImpl<_$CardValue_String>(this, _$identity);

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
  const factory CardValue_String({required final String value}) = _$CardValue_String;

  @override
  String get value;
  @JsonKey(ignore: true)
  _$$CardValue_StringCopyWith<_$CardValue_String> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$CardValue_BooleanCopyWith<$Res> {
  factory _$$CardValue_BooleanCopyWith(_$CardValue_Boolean value, $Res Function(_$CardValue_Boolean) then) =
      __$$CardValue_BooleanCopyWithImpl<$Res>;
  @useResult
  $Res call({bool value});
}

/// @nodoc
class __$$CardValue_BooleanCopyWithImpl<$Res> extends _$CardValueCopyWithImpl<$Res, _$CardValue_Boolean>
    implements _$$CardValue_BooleanCopyWith<$Res> {
  __$$CardValue_BooleanCopyWithImpl(_$CardValue_Boolean _value, $Res Function(_$CardValue_Boolean) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$CardValue_Boolean(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc

class _$CardValue_Boolean implements CardValue_Boolean {
  const _$CardValue_Boolean({required this.value});

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
            other is _$CardValue_Boolean &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$CardValue_BooleanCopyWith<_$CardValue_Boolean> get copyWith =>
      __$$CardValue_BooleanCopyWithImpl<_$CardValue_Boolean>(this, _$identity);

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
  const factory CardValue_Boolean({required final bool value}) = _$CardValue_Boolean;

  @override
  bool get value;
  @JsonKey(ignore: true)
  _$$CardValue_BooleanCopyWith<_$CardValue_Boolean> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$CardValue_DateCopyWith<$Res> {
  factory _$$CardValue_DateCopyWith(_$CardValue_Date value, $Res Function(_$CardValue_Date) then) =
      __$$CardValue_DateCopyWithImpl<$Res>;
  @useResult
  $Res call({String value});
}

/// @nodoc
class __$$CardValue_DateCopyWithImpl<$Res> extends _$CardValueCopyWithImpl<$Res, _$CardValue_Date>
    implements _$$CardValue_DateCopyWith<$Res> {
  __$$CardValue_DateCopyWithImpl(_$CardValue_Date _value, $Res Function(_$CardValue_Date) _then) : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$CardValue_Date(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$CardValue_Date implements CardValue_Date {
  const _$CardValue_Date({required this.value});

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
            other is _$CardValue_Date &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$CardValue_DateCopyWith<_$CardValue_Date> get copyWith =>
      __$$CardValue_DateCopyWithImpl<_$CardValue_Date>(this, _$identity);

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
  const factory CardValue_Date({required final String value}) = _$CardValue_Date;

  @override
  String get value;
  @JsonKey(ignore: true)
  _$$CardValue_DateCopyWith<_$CardValue_Date> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$CardValue_GenderCopyWith<$Res> {
  factory _$$CardValue_GenderCopyWith(_$CardValue_Gender value, $Res Function(_$CardValue_Gender) then) =
      __$$CardValue_GenderCopyWithImpl<$Res>;
  @useResult
  $Res call({GenderCardValue value});
}

/// @nodoc
class __$$CardValue_GenderCopyWithImpl<$Res> extends _$CardValueCopyWithImpl<$Res, _$CardValue_Gender>
    implements _$$CardValue_GenderCopyWith<$Res> {
  __$$CardValue_GenderCopyWithImpl(_$CardValue_Gender _value, $Res Function(_$CardValue_Gender) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$CardValue_Gender(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as GenderCardValue,
    ));
  }
}

/// @nodoc

class _$CardValue_Gender implements CardValue_Gender {
  const _$CardValue_Gender({required this.value});

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
            other is _$CardValue_Gender &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$CardValue_GenderCopyWith<_$CardValue_Gender> get copyWith =>
      __$$CardValue_GenderCopyWithImpl<_$CardValue_Gender>(this, _$identity);

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
  const factory CardValue_Gender({required final GenderCardValue value}) = _$CardValue_Gender;

  @override
  GenderCardValue get value;
  @JsonKey(ignore: true)
  _$$CardValue_GenderCopyWith<_$CardValue_Gender> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$DisclosureEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() fetchingRequest,
    required TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards) request,
    required TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)
        requestAttributesMissing,
    required TResult Function(String data) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? fetchingRequest,
    TResult? Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult? Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult? Function(String data)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? fetchingRequest,
    TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult Function(String data)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(DisclosureEvent_FetchingRequest value) fetchingRequest,
    required TResult Function(DisclosureEvent_Request value) request,
    required TResult Function(DisclosureEvent_RequestAttributesMissing value) requestAttributesMissing,
    required TResult Function(DisclosureEvent_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult? Function(DisclosureEvent_Request value)? request,
    TResult? Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult? Function(DisclosureEvent_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult Function(DisclosureEvent_Request value)? request,
    TResult Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult Function(DisclosureEvent_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DisclosureEventCopyWith<$Res> {
  factory $DisclosureEventCopyWith(DisclosureEvent value, $Res Function(DisclosureEvent) then) =
      _$DisclosureEventCopyWithImpl<$Res, DisclosureEvent>;
}

/// @nodoc
class _$DisclosureEventCopyWithImpl<$Res, $Val extends DisclosureEvent> implements $DisclosureEventCopyWith<$Res> {
  _$DisclosureEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;
}

/// @nodoc
abstract class _$$DisclosureEvent_FetchingRequestCopyWith<$Res> {
  factory _$$DisclosureEvent_FetchingRequestCopyWith(
          _$DisclosureEvent_FetchingRequest value, $Res Function(_$DisclosureEvent_FetchingRequest) then) =
      __$$DisclosureEvent_FetchingRequestCopyWithImpl<$Res>;
}

/// @nodoc
class __$$DisclosureEvent_FetchingRequestCopyWithImpl<$Res>
    extends _$DisclosureEventCopyWithImpl<$Res, _$DisclosureEvent_FetchingRequest>
    implements _$$DisclosureEvent_FetchingRequestCopyWith<$Res> {
  __$$DisclosureEvent_FetchingRequestCopyWithImpl(
      _$DisclosureEvent_FetchingRequest _value, $Res Function(_$DisclosureEvent_FetchingRequest) _then)
      : super(_value, _then);
}

/// @nodoc

class _$DisclosureEvent_FetchingRequest implements DisclosureEvent_FetchingRequest {
  const _$DisclosureEvent_FetchingRequest();

  @override
  String toString() {
    return 'DisclosureEvent.fetchingRequest()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$DisclosureEvent_FetchingRequest);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() fetchingRequest,
    required TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards) request,
    required TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)
        requestAttributesMissing,
    required TResult Function(String data) error,
  }) {
    return fetchingRequest();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? fetchingRequest,
    TResult? Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult? Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult? Function(String data)? error,
  }) {
    return fetchingRequest?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? fetchingRequest,
    TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult Function(String data)? error,
    required TResult orElse(),
  }) {
    if (fetchingRequest != null) {
      return fetchingRequest();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(DisclosureEvent_FetchingRequest value) fetchingRequest,
    required TResult Function(DisclosureEvent_Request value) request,
    required TResult Function(DisclosureEvent_RequestAttributesMissing value) requestAttributesMissing,
    required TResult Function(DisclosureEvent_Error value) error,
  }) {
    return fetchingRequest(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult? Function(DisclosureEvent_Request value)? request,
    TResult? Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult? Function(DisclosureEvent_Error value)? error,
  }) {
    return fetchingRequest?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult Function(DisclosureEvent_Request value)? request,
    TResult Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult Function(DisclosureEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (fetchingRequest != null) {
      return fetchingRequest(this);
    }
    return orElse();
  }
}

abstract class DisclosureEvent_FetchingRequest implements DisclosureEvent {
  const factory DisclosureEvent_FetchingRequest() = _$DisclosureEvent_FetchingRequest;
}

/// @nodoc
abstract class _$$DisclosureEvent_RequestCopyWith<$Res> {
  factory _$$DisclosureEvent_RequestCopyWith(
          _$DisclosureEvent_Request value, $Res Function(_$DisclosureEvent_Request) then) =
      __$$DisclosureEvent_RequestCopyWithImpl<$Res>;
  @useResult
  $Res call({RelyingParty relyingParty, List<RequestedCard> requestedCards});
}

/// @nodoc
class __$$DisclosureEvent_RequestCopyWithImpl<$Res>
    extends _$DisclosureEventCopyWithImpl<$Res, _$DisclosureEvent_Request>
    implements _$$DisclosureEvent_RequestCopyWith<$Res> {
  __$$DisclosureEvent_RequestCopyWithImpl(
      _$DisclosureEvent_Request _value, $Res Function(_$DisclosureEvent_Request) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? requestedCards = null,
  }) {
    return _then(_$DisclosureEvent_Request(
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

class _$DisclosureEvent_Request implements DisclosureEvent_Request {
  const _$DisclosureEvent_Request({required this.relyingParty, required final List<RequestedCard> requestedCards})
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
    return 'DisclosureEvent.request(relyingParty: $relyingParty, requestedCards: $requestedCards)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DisclosureEvent_Request &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._requestedCards, _requestedCards));
  }

  @override
  int get hashCode => Object.hash(runtimeType, relyingParty, const DeepCollectionEquality().hash(_requestedCards));

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$DisclosureEvent_RequestCopyWith<_$DisclosureEvent_Request> get copyWith =>
      __$$DisclosureEvent_RequestCopyWithImpl<_$DisclosureEvent_Request>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() fetchingRequest,
    required TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards) request,
    required TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)
        requestAttributesMissing,
    required TResult Function(String data) error,
  }) {
    return request(relyingParty, requestedCards);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? fetchingRequest,
    TResult? Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult? Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult? Function(String data)? error,
  }) {
    return request?.call(relyingParty, requestedCards);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? fetchingRequest,
    TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult Function(String data)? error,
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
    required TResult Function(DisclosureEvent_FetchingRequest value) fetchingRequest,
    required TResult Function(DisclosureEvent_Request value) request,
    required TResult Function(DisclosureEvent_RequestAttributesMissing value) requestAttributesMissing,
    required TResult Function(DisclosureEvent_Error value) error,
  }) {
    return request(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult? Function(DisclosureEvent_Request value)? request,
    TResult? Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult? Function(DisclosureEvent_Error value)? error,
  }) {
    return request?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult Function(DisclosureEvent_Request value)? request,
    TResult Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult Function(DisclosureEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (request != null) {
      return request(this);
    }
    return orElse();
  }
}

abstract class DisclosureEvent_Request implements DisclosureEvent {
  const factory DisclosureEvent_Request(
      {required final RelyingParty relyingParty,
      required final List<RequestedCard> requestedCards}) = _$DisclosureEvent_Request;

  RelyingParty get relyingParty;
  List<RequestedCard> get requestedCards;
  @JsonKey(ignore: true)
  _$$DisclosureEvent_RequestCopyWith<_$DisclosureEvent_Request> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$DisclosureEvent_RequestAttributesMissingCopyWith<$Res> {
  factory _$$DisclosureEvent_RequestAttributesMissingCopyWith(_$DisclosureEvent_RequestAttributesMissing value,
          $Res Function(_$DisclosureEvent_RequestAttributesMissing) then) =
      __$$DisclosureEvent_RequestAttributesMissingCopyWithImpl<$Res>;
  @useResult
  $Res call({RelyingParty relyingParty, List<MissingAttribute> missingAttributes});
}

/// @nodoc
class __$$DisclosureEvent_RequestAttributesMissingCopyWithImpl<$Res>
    extends _$DisclosureEventCopyWithImpl<$Res, _$DisclosureEvent_RequestAttributesMissing>
    implements _$$DisclosureEvent_RequestAttributesMissingCopyWith<$Res> {
  __$$DisclosureEvent_RequestAttributesMissingCopyWithImpl(_$DisclosureEvent_RequestAttributesMissing _value,
      $Res Function(_$DisclosureEvent_RequestAttributesMissing) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? missingAttributes = null,
  }) {
    return _then(_$DisclosureEvent_RequestAttributesMissing(
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

class _$DisclosureEvent_RequestAttributesMissing implements DisclosureEvent_RequestAttributesMissing {
  const _$DisclosureEvent_RequestAttributesMissing(
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
    return 'DisclosureEvent.requestAttributesMissing(relyingParty: $relyingParty, missingAttributes: $missingAttributes)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DisclosureEvent_RequestAttributesMissing &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._missingAttributes, _missingAttributes));
  }

  @override
  int get hashCode => Object.hash(runtimeType, relyingParty, const DeepCollectionEquality().hash(_missingAttributes));

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$DisclosureEvent_RequestAttributesMissingCopyWith<_$DisclosureEvent_RequestAttributesMissing> get copyWith =>
      __$$DisclosureEvent_RequestAttributesMissingCopyWithImpl<_$DisclosureEvent_RequestAttributesMissing>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() fetchingRequest,
    required TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards) request,
    required TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)
        requestAttributesMissing,
    required TResult Function(String data) error,
  }) {
    return requestAttributesMissing(relyingParty, missingAttributes);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? fetchingRequest,
    TResult? Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult? Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult? Function(String data)? error,
  }) {
    return requestAttributesMissing?.call(relyingParty, missingAttributes);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? fetchingRequest,
    TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult Function(String data)? error,
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
    required TResult Function(DisclosureEvent_FetchingRequest value) fetchingRequest,
    required TResult Function(DisclosureEvent_Request value) request,
    required TResult Function(DisclosureEvent_RequestAttributesMissing value) requestAttributesMissing,
    required TResult Function(DisclosureEvent_Error value) error,
  }) {
    return requestAttributesMissing(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult? Function(DisclosureEvent_Request value)? request,
    TResult? Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult? Function(DisclosureEvent_Error value)? error,
  }) {
    return requestAttributesMissing?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult Function(DisclosureEvent_Request value)? request,
    TResult Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult Function(DisclosureEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (requestAttributesMissing != null) {
      return requestAttributesMissing(this);
    }
    return orElse();
  }
}

abstract class DisclosureEvent_RequestAttributesMissing implements DisclosureEvent {
  const factory DisclosureEvent_RequestAttributesMissing(
      {required final RelyingParty relyingParty,
      required final List<MissingAttribute> missingAttributes}) = _$DisclosureEvent_RequestAttributesMissing;

  RelyingParty get relyingParty;
  List<MissingAttribute> get missingAttributes;
  @JsonKey(ignore: true)
  _$$DisclosureEvent_RequestAttributesMissingCopyWith<_$DisclosureEvent_RequestAttributesMissing> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$DisclosureEvent_ErrorCopyWith<$Res> {
  factory _$$DisclosureEvent_ErrorCopyWith(_$DisclosureEvent_Error value, $Res Function(_$DisclosureEvent_Error) then) =
      __$$DisclosureEvent_ErrorCopyWithImpl<$Res>;
  @useResult
  $Res call({String data});
}

/// @nodoc
class __$$DisclosureEvent_ErrorCopyWithImpl<$Res> extends _$DisclosureEventCopyWithImpl<$Res, _$DisclosureEvent_Error>
    implements _$$DisclosureEvent_ErrorCopyWith<$Res> {
  __$$DisclosureEvent_ErrorCopyWithImpl(_$DisclosureEvent_Error _value, $Res Function(_$DisclosureEvent_Error) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? data = null,
  }) {
    return _then(_$DisclosureEvent_Error(
      data: null == data
          ? _value.data
          : data // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$DisclosureEvent_Error implements DisclosureEvent_Error {
  const _$DisclosureEvent_Error({required this.data});

  @override
  final String data;

  @override
  String toString() {
    return 'DisclosureEvent.error(data: $data)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DisclosureEvent_Error &&
            (identical(other.data, data) || other.data == data));
  }

  @override
  int get hashCode => Object.hash(runtimeType, data);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$DisclosureEvent_ErrorCopyWith<_$DisclosureEvent_Error> get copyWith =>
      __$$DisclosureEvent_ErrorCopyWithImpl<_$DisclosureEvent_Error>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() fetchingRequest,
    required TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards) request,
    required TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)
        requestAttributesMissing,
    required TResult Function(String data) error,
  }) {
    return error(data);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? fetchingRequest,
    TResult? Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult? Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult? Function(String data)? error,
  }) {
    return error?.call(data);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? fetchingRequest,
    TResult Function(RelyingParty relyingParty, List<RequestedCard> requestedCards)? request,
    TResult Function(RelyingParty relyingParty, List<MissingAttribute> missingAttributes)? requestAttributesMissing,
    TResult Function(String data)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(data);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(DisclosureEvent_FetchingRequest value) fetchingRequest,
    required TResult Function(DisclosureEvent_Request value) request,
    required TResult Function(DisclosureEvent_RequestAttributesMissing value) requestAttributesMissing,
    required TResult Function(DisclosureEvent_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult? Function(DisclosureEvent_Request value)? request,
    TResult? Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult? Function(DisclosureEvent_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DisclosureEvent_FetchingRequest value)? fetchingRequest,
    TResult Function(DisclosureEvent_Request value)? request,
    TResult Function(DisclosureEvent_RequestAttributesMissing value)? requestAttributesMissing,
    TResult Function(DisclosureEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class DisclosureEvent_Error implements DisclosureEvent {
  const factory DisclosureEvent_Error({required final String data}) = _$DisclosureEvent_Error;

  String get data;
  @JsonKey(ignore: true)
  _$$DisclosureEvent_ErrorCopyWith<_$DisclosureEvent_Error> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$PidIssuanceEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() authenticating,
    required TResult Function(List<Card> previewCards) success,
    required TResult Function(String data) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? authenticating,
    TResult? Function(List<Card> previewCards)? success,
    TResult? Function(String data)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? authenticating,
    TResult Function(List<Card> previewCards)? success,
    TResult Function(String data)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PidIssuanceEvent_Authenticating value) authenticating,
    required TResult Function(PidIssuanceEvent_Success value) success,
    required TResult Function(PidIssuanceEvent_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PidIssuanceEvent_Authenticating value)? authenticating,
    TResult? Function(PidIssuanceEvent_Success value)? success,
    TResult? Function(PidIssuanceEvent_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PidIssuanceEvent_Authenticating value)? authenticating,
    TResult Function(PidIssuanceEvent_Success value)? success,
    TResult Function(PidIssuanceEvent_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PidIssuanceEventCopyWith<$Res> {
  factory $PidIssuanceEventCopyWith(PidIssuanceEvent value, $Res Function(PidIssuanceEvent) then) =
      _$PidIssuanceEventCopyWithImpl<$Res, PidIssuanceEvent>;
}

/// @nodoc
class _$PidIssuanceEventCopyWithImpl<$Res, $Val extends PidIssuanceEvent> implements $PidIssuanceEventCopyWith<$Res> {
  _$PidIssuanceEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;
}

/// @nodoc
abstract class _$$PidIssuanceEvent_AuthenticatingCopyWith<$Res> {
  factory _$$PidIssuanceEvent_AuthenticatingCopyWith(
          _$PidIssuanceEvent_Authenticating value, $Res Function(_$PidIssuanceEvent_Authenticating) then) =
      __$$PidIssuanceEvent_AuthenticatingCopyWithImpl<$Res>;
}

/// @nodoc
class __$$PidIssuanceEvent_AuthenticatingCopyWithImpl<$Res>
    extends _$PidIssuanceEventCopyWithImpl<$Res, _$PidIssuanceEvent_Authenticating>
    implements _$$PidIssuanceEvent_AuthenticatingCopyWith<$Res> {
  __$$PidIssuanceEvent_AuthenticatingCopyWithImpl(
      _$PidIssuanceEvent_Authenticating _value, $Res Function(_$PidIssuanceEvent_Authenticating) _then)
      : super(_value, _then);
}

/// @nodoc

class _$PidIssuanceEvent_Authenticating implements PidIssuanceEvent_Authenticating {
  const _$PidIssuanceEvent_Authenticating();

  @override
  String toString() {
    return 'PidIssuanceEvent.authenticating()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$PidIssuanceEvent_Authenticating);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() authenticating,
    required TResult Function(List<Card> previewCards) success,
    required TResult Function(String data) error,
  }) {
    return authenticating();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? authenticating,
    TResult? Function(List<Card> previewCards)? success,
    TResult? Function(String data)? error,
  }) {
    return authenticating?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? authenticating,
    TResult Function(List<Card> previewCards)? success,
    TResult Function(String data)? error,
    required TResult orElse(),
  }) {
    if (authenticating != null) {
      return authenticating();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PidIssuanceEvent_Authenticating value) authenticating,
    required TResult Function(PidIssuanceEvent_Success value) success,
    required TResult Function(PidIssuanceEvent_Error value) error,
  }) {
    return authenticating(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PidIssuanceEvent_Authenticating value)? authenticating,
    TResult? Function(PidIssuanceEvent_Success value)? success,
    TResult? Function(PidIssuanceEvent_Error value)? error,
  }) {
    return authenticating?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PidIssuanceEvent_Authenticating value)? authenticating,
    TResult Function(PidIssuanceEvent_Success value)? success,
    TResult Function(PidIssuanceEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (authenticating != null) {
      return authenticating(this);
    }
    return orElse();
  }
}

abstract class PidIssuanceEvent_Authenticating implements PidIssuanceEvent {
  const factory PidIssuanceEvent_Authenticating() = _$PidIssuanceEvent_Authenticating;
}

/// @nodoc
abstract class _$$PidIssuanceEvent_SuccessCopyWith<$Res> {
  factory _$$PidIssuanceEvent_SuccessCopyWith(
          _$PidIssuanceEvent_Success value, $Res Function(_$PidIssuanceEvent_Success) then) =
      __$$PidIssuanceEvent_SuccessCopyWithImpl<$Res>;
  @useResult
  $Res call({List<Card> previewCards});
}

/// @nodoc
class __$$PidIssuanceEvent_SuccessCopyWithImpl<$Res>
    extends _$PidIssuanceEventCopyWithImpl<$Res, _$PidIssuanceEvent_Success>
    implements _$$PidIssuanceEvent_SuccessCopyWith<$Res> {
  __$$PidIssuanceEvent_SuccessCopyWithImpl(
      _$PidIssuanceEvent_Success _value, $Res Function(_$PidIssuanceEvent_Success) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? previewCards = null,
  }) {
    return _then(_$PidIssuanceEvent_Success(
      previewCards: null == previewCards
          ? _value._previewCards
          : previewCards // ignore: cast_nullable_to_non_nullable
              as List<Card>,
    ));
  }
}

/// @nodoc

class _$PidIssuanceEvent_Success implements PidIssuanceEvent_Success {
  const _$PidIssuanceEvent_Success({required final List<Card> previewCards}) : _previewCards = previewCards;

  final List<Card> _previewCards;
  @override
  List<Card> get previewCards {
    if (_previewCards is EqualUnmodifiableListView) return _previewCards;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_previewCards);
  }

  @override
  String toString() {
    return 'PidIssuanceEvent.success(previewCards: $previewCards)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PidIssuanceEvent_Success &&
            const DeepCollectionEquality().equals(other._previewCards, _previewCards));
  }

  @override
  int get hashCode => Object.hash(runtimeType, const DeepCollectionEquality().hash(_previewCards));

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$PidIssuanceEvent_SuccessCopyWith<_$PidIssuanceEvent_Success> get copyWith =>
      __$$PidIssuanceEvent_SuccessCopyWithImpl<_$PidIssuanceEvent_Success>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() authenticating,
    required TResult Function(List<Card> previewCards) success,
    required TResult Function(String data) error,
  }) {
    return success(previewCards);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? authenticating,
    TResult? Function(List<Card> previewCards)? success,
    TResult? Function(String data)? error,
  }) {
    return success?.call(previewCards);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? authenticating,
    TResult Function(List<Card> previewCards)? success,
    TResult Function(String data)? error,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(previewCards);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PidIssuanceEvent_Authenticating value) authenticating,
    required TResult Function(PidIssuanceEvent_Success value) success,
    required TResult Function(PidIssuanceEvent_Error value) error,
  }) {
    return success(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PidIssuanceEvent_Authenticating value)? authenticating,
    TResult? Function(PidIssuanceEvent_Success value)? success,
    TResult? Function(PidIssuanceEvent_Error value)? error,
  }) {
    return success?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PidIssuanceEvent_Authenticating value)? authenticating,
    TResult Function(PidIssuanceEvent_Success value)? success,
    TResult Function(PidIssuanceEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(this);
    }
    return orElse();
  }
}

abstract class PidIssuanceEvent_Success implements PidIssuanceEvent {
  const factory PidIssuanceEvent_Success({required final List<Card> previewCards}) = _$PidIssuanceEvent_Success;

  List<Card> get previewCards;
  @JsonKey(ignore: true)
  _$$PidIssuanceEvent_SuccessCopyWith<_$PidIssuanceEvent_Success> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PidIssuanceEvent_ErrorCopyWith<$Res> {
  factory _$$PidIssuanceEvent_ErrorCopyWith(
          _$PidIssuanceEvent_Error value, $Res Function(_$PidIssuanceEvent_Error) then) =
      __$$PidIssuanceEvent_ErrorCopyWithImpl<$Res>;
  @useResult
  $Res call({String data});
}

/// @nodoc
class __$$PidIssuanceEvent_ErrorCopyWithImpl<$Res>
    extends _$PidIssuanceEventCopyWithImpl<$Res, _$PidIssuanceEvent_Error>
    implements _$$PidIssuanceEvent_ErrorCopyWith<$Res> {
  __$$PidIssuanceEvent_ErrorCopyWithImpl(_$PidIssuanceEvent_Error _value, $Res Function(_$PidIssuanceEvent_Error) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? data = null,
  }) {
    return _then(_$PidIssuanceEvent_Error(
      data: null == data
          ? _value.data
          : data // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$PidIssuanceEvent_Error implements PidIssuanceEvent_Error {
  const _$PidIssuanceEvent_Error({required this.data});

  @override
  final String data;

  @override
  String toString() {
    return 'PidIssuanceEvent.error(data: $data)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PidIssuanceEvent_Error &&
            (identical(other.data, data) || other.data == data));
  }

  @override
  int get hashCode => Object.hash(runtimeType, data);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$PidIssuanceEvent_ErrorCopyWith<_$PidIssuanceEvent_Error> get copyWith =>
      __$$PidIssuanceEvent_ErrorCopyWithImpl<_$PidIssuanceEvent_Error>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() authenticating,
    required TResult Function(List<Card> previewCards) success,
    required TResult Function(String data) error,
  }) {
    return error(data);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? authenticating,
    TResult? Function(List<Card> previewCards)? success,
    TResult? Function(String data)? error,
  }) {
    return error?.call(data);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? authenticating,
    TResult Function(List<Card> previewCards)? success,
    TResult Function(String data)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(data);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PidIssuanceEvent_Authenticating value) authenticating,
    required TResult Function(PidIssuanceEvent_Success value) success,
    required TResult Function(PidIssuanceEvent_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PidIssuanceEvent_Authenticating value)? authenticating,
    TResult? Function(PidIssuanceEvent_Success value)? success,
    TResult? Function(PidIssuanceEvent_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PidIssuanceEvent_Authenticating value)? authenticating,
    TResult Function(PidIssuanceEvent_Success value)? success,
    TResult Function(PidIssuanceEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class PidIssuanceEvent_Error implements PidIssuanceEvent {
  const factory PidIssuanceEvent_Error({required final String data}) = _$PidIssuanceEvent_Error;

  String get data;
  @JsonKey(ignore: true)
  _$$PidIssuanceEvent_ErrorCopyWith<_$PidIssuanceEvent_Error> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$ProcessUriEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PidIssuanceEvent event) pidIssuance,
    required TResult Function(DisclosureEvent event) disclosure,
    required TResult Function() unknownUri,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PidIssuanceEvent event)? pidIssuance,
    TResult? Function(DisclosureEvent event)? disclosure,
    TResult? Function()? unknownUri,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PidIssuanceEvent event)? pidIssuance,
    TResult Function(DisclosureEvent event)? disclosure,
    TResult Function()? unknownUri,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ProcessUriEvent_PidIssuance value) pidIssuance,
    required TResult Function(ProcessUriEvent_Disclosure value) disclosure,
    required TResult Function(ProcessUriEvent_UnknownUri value) unknownUri,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ProcessUriEvent_PidIssuance value)? pidIssuance,
    TResult? Function(ProcessUriEvent_Disclosure value)? disclosure,
    TResult? Function(ProcessUriEvent_UnknownUri value)? unknownUri,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ProcessUriEvent_PidIssuance value)? pidIssuance,
    TResult Function(ProcessUriEvent_Disclosure value)? disclosure,
    TResult Function(ProcessUriEvent_UnknownUri value)? unknownUri,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ProcessUriEventCopyWith<$Res> {
  factory $ProcessUriEventCopyWith(ProcessUriEvent value, $Res Function(ProcessUriEvent) then) =
      _$ProcessUriEventCopyWithImpl<$Res, ProcessUriEvent>;
}

/// @nodoc
class _$ProcessUriEventCopyWithImpl<$Res, $Val extends ProcessUriEvent> implements $ProcessUriEventCopyWith<$Res> {
  _$ProcessUriEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;
}

/// @nodoc
abstract class _$$ProcessUriEvent_PidIssuanceCopyWith<$Res> {
  factory _$$ProcessUriEvent_PidIssuanceCopyWith(
          _$ProcessUriEvent_PidIssuance value, $Res Function(_$ProcessUriEvent_PidIssuance) then) =
      __$$ProcessUriEvent_PidIssuanceCopyWithImpl<$Res>;
  @useResult
  $Res call({PidIssuanceEvent event});

  $PidIssuanceEventCopyWith<$Res> get event;
}

/// @nodoc
class __$$ProcessUriEvent_PidIssuanceCopyWithImpl<$Res>
    extends _$ProcessUriEventCopyWithImpl<$Res, _$ProcessUriEvent_PidIssuance>
    implements _$$ProcessUriEvent_PidIssuanceCopyWith<$Res> {
  __$$ProcessUriEvent_PidIssuanceCopyWithImpl(
      _$ProcessUriEvent_PidIssuance _value, $Res Function(_$ProcessUriEvent_PidIssuance) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? event = null,
  }) {
    return _then(_$ProcessUriEvent_PidIssuance(
      event: null == event
          ? _value.event
          : event // ignore: cast_nullable_to_non_nullable
              as PidIssuanceEvent,
    ));
  }

  @override
  @pragma('vm:prefer-inline')
  $PidIssuanceEventCopyWith<$Res> get event {
    return $PidIssuanceEventCopyWith<$Res>(_value.event, (value) {
      return _then(_value.copyWith(event: value));
    });
  }
}

/// @nodoc

class _$ProcessUriEvent_PidIssuance implements ProcessUriEvent_PidIssuance {
  const _$ProcessUriEvent_PidIssuance({required this.event});

  @override
  final PidIssuanceEvent event;

  @override
  String toString() {
    return 'ProcessUriEvent.pidIssuance(event: $event)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ProcessUriEvent_PidIssuance &&
            (identical(other.event, event) || other.event == event));
  }

  @override
  int get hashCode => Object.hash(runtimeType, event);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$ProcessUriEvent_PidIssuanceCopyWith<_$ProcessUriEvent_PidIssuance> get copyWith =>
      __$$ProcessUriEvent_PidIssuanceCopyWithImpl<_$ProcessUriEvent_PidIssuance>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PidIssuanceEvent event) pidIssuance,
    required TResult Function(DisclosureEvent event) disclosure,
    required TResult Function() unknownUri,
  }) {
    return pidIssuance(event);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PidIssuanceEvent event)? pidIssuance,
    TResult? Function(DisclosureEvent event)? disclosure,
    TResult? Function()? unknownUri,
  }) {
    return pidIssuance?.call(event);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PidIssuanceEvent event)? pidIssuance,
    TResult Function(DisclosureEvent event)? disclosure,
    TResult Function()? unknownUri,
    required TResult orElse(),
  }) {
    if (pidIssuance != null) {
      return pidIssuance(event);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ProcessUriEvent_PidIssuance value) pidIssuance,
    required TResult Function(ProcessUriEvent_Disclosure value) disclosure,
    required TResult Function(ProcessUriEvent_UnknownUri value) unknownUri,
  }) {
    return pidIssuance(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ProcessUriEvent_PidIssuance value)? pidIssuance,
    TResult? Function(ProcessUriEvent_Disclosure value)? disclosure,
    TResult? Function(ProcessUriEvent_UnknownUri value)? unknownUri,
  }) {
    return pidIssuance?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ProcessUriEvent_PidIssuance value)? pidIssuance,
    TResult Function(ProcessUriEvent_Disclosure value)? disclosure,
    TResult Function(ProcessUriEvent_UnknownUri value)? unknownUri,
    required TResult orElse(),
  }) {
    if (pidIssuance != null) {
      return pidIssuance(this);
    }
    return orElse();
  }
}

abstract class ProcessUriEvent_PidIssuance implements ProcessUriEvent {
  const factory ProcessUriEvent_PidIssuance({required final PidIssuanceEvent event}) = _$ProcessUriEvent_PidIssuance;

  PidIssuanceEvent get event;
  @JsonKey(ignore: true)
  _$$ProcessUriEvent_PidIssuanceCopyWith<_$ProcessUriEvent_PidIssuance> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ProcessUriEvent_DisclosureCopyWith<$Res> {
  factory _$$ProcessUriEvent_DisclosureCopyWith(
          _$ProcessUriEvent_Disclosure value, $Res Function(_$ProcessUriEvent_Disclosure) then) =
      __$$ProcessUriEvent_DisclosureCopyWithImpl<$Res>;
  @useResult
  $Res call({DisclosureEvent event});

  $DisclosureEventCopyWith<$Res> get event;
}

/// @nodoc
class __$$ProcessUriEvent_DisclosureCopyWithImpl<$Res>
    extends _$ProcessUriEventCopyWithImpl<$Res, _$ProcessUriEvent_Disclosure>
    implements _$$ProcessUriEvent_DisclosureCopyWith<$Res> {
  __$$ProcessUriEvent_DisclosureCopyWithImpl(
      _$ProcessUriEvent_Disclosure _value, $Res Function(_$ProcessUriEvent_Disclosure) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? event = null,
  }) {
    return _then(_$ProcessUriEvent_Disclosure(
      event: null == event
          ? _value.event
          : event // ignore: cast_nullable_to_non_nullable
              as DisclosureEvent,
    ));
  }

  @override
  @pragma('vm:prefer-inline')
  $DisclosureEventCopyWith<$Res> get event {
    return $DisclosureEventCopyWith<$Res>(_value.event, (value) {
      return _then(_value.copyWith(event: value));
    });
  }
}

/// @nodoc

class _$ProcessUriEvent_Disclosure implements ProcessUriEvent_Disclosure {
  const _$ProcessUriEvent_Disclosure({required this.event});

  @override
  final DisclosureEvent event;

  @override
  String toString() {
    return 'ProcessUriEvent.disclosure(event: $event)';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ProcessUriEvent_Disclosure &&
            (identical(other.event, event) || other.event == event));
  }

  @override
  int get hashCode => Object.hash(runtimeType, event);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$ProcessUriEvent_DisclosureCopyWith<_$ProcessUriEvent_Disclosure> get copyWith =>
      __$$ProcessUriEvent_DisclosureCopyWithImpl<_$ProcessUriEvent_Disclosure>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PidIssuanceEvent event) pidIssuance,
    required TResult Function(DisclosureEvent event) disclosure,
    required TResult Function() unknownUri,
  }) {
    return disclosure(event);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PidIssuanceEvent event)? pidIssuance,
    TResult? Function(DisclosureEvent event)? disclosure,
    TResult? Function()? unknownUri,
  }) {
    return disclosure?.call(event);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PidIssuanceEvent event)? pidIssuance,
    TResult Function(DisclosureEvent event)? disclosure,
    TResult Function()? unknownUri,
    required TResult orElse(),
  }) {
    if (disclosure != null) {
      return disclosure(event);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ProcessUriEvent_PidIssuance value) pidIssuance,
    required TResult Function(ProcessUriEvent_Disclosure value) disclosure,
    required TResult Function(ProcessUriEvent_UnknownUri value) unknownUri,
  }) {
    return disclosure(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ProcessUriEvent_PidIssuance value)? pidIssuance,
    TResult? Function(ProcessUriEvent_Disclosure value)? disclosure,
    TResult? Function(ProcessUriEvent_UnknownUri value)? unknownUri,
  }) {
    return disclosure?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ProcessUriEvent_PidIssuance value)? pidIssuance,
    TResult Function(ProcessUriEvent_Disclosure value)? disclosure,
    TResult Function(ProcessUriEvent_UnknownUri value)? unknownUri,
    required TResult orElse(),
  }) {
    if (disclosure != null) {
      return disclosure(this);
    }
    return orElse();
  }
}

abstract class ProcessUriEvent_Disclosure implements ProcessUriEvent {
  const factory ProcessUriEvent_Disclosure({required final DisclosureEvent event}) = _$ProcessUriEvent_Disclosure;

  DisclosureEvent get event;
  @JsonKey(ignore: true)
  _$$ProcessUriEvent_DisclosureCopyWith<_$ProcessUriEvent_Disclosure> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ProcessUriEvent_UnknownUriCopyWith<$Res> {
  factory _$$ProcessUriEvent_UnknownUriCopyWith(
          _$ProcessUriEvent_UnknownUri value, $Res Function(_$ProcessUriEvent_UnknownUri) then) =
      __$$ProcessUriEvent_UnknownUriCopyWithImpl<$Res>;
}

/// @nodoc
class __$$ProcessUriEvent_UnknownUriCopyWithImpl<$Res>
    extends _$ProcessUriEventCopyWithImpl<$Res, _$ProcessUriEvent_UnknownUri>
    implements _$$ProcessUriEvent_UnknownUriCopyWith<$Res> {
  __$$ProcessUriEvent_UnknownUriCopyWithImpl(
      _$ProcessUriEvent_UnknownUri _value, $Res Function(_$ProcessUriEvent_UnknownUri) _then)
      : super(_value, _then);
}

/// @nodoc

class _$ProcessUriEvent_UnknownUri implements ProcessUriEvent_UnknownUri {
  const _$ProcessUriEvent_UnknownUri();

  @override
  String toString() {
    return 'ProcessUriEvent.unknownUri()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$ProcessUriEvent_UnknownUri);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PidIssuanceEvent event) pidIssuance,
    required TResult Function(DisclosureEvent event) disclosure,
    required TResult Function() unknownUri,
  }) {
    return unknownUri();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PidIssuanceEvent event)? pidIssuance,
    TResult? Function(DisclosureEvent event)? disclosure,
    TResult? Function()? unknownUri,
  }) {
    return unknownUri?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PidIssuanceEvent event)? pidIssuance,
    TResult Function(DisclosureEvent event)? disclosure,
    TResult Function()? unknownUri,
    required TResult orElse(),
  }) {
    if (unknownUri != null) {
      return unknownUri();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ProcessUriEvent_PidIssuance value) pidIssuance,
    required TResult Function(ProcessUriEvent_Disclosure value) disclosure,
    required TResult Function(ProcessUriEvent_UnknownUri value) unknownUri,
  }) {
    return unknownUri(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ProcessUriEvent_PidIssuance value)? pidIssuance,
    TResult? Function(ProcessUriEvent_Disclosure value)? disclosure,
    TResult? Function(ProcessUriEvent_UnknownUri value)? unknownUri,
  }) {
    return unknownUri?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ProcessUriEvent_PidIssuance value)? pidIssuance,
    TResult Function(ProcessUriEvent_Disclosure value)? disclosure,
    TResult Function(ProcessUriEvent_UnknownUri value)? unknownUri,
    required TResult orElse(),
  }) {
    if (unknownUri != null) {
      return unknownUri(this);
    }
    return orElse();
  }
}

abstract class ProcessUriEvent_UnknownUri implements ProcessUriEvent {
  const factory ProcessUriEvent_UnknownUri() = _$ProcessUriEvent_UnknownUri;
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
abstract class _$$WalletInstructionResult_OkCopyWith<$Res> {
  factory _$$WalletInstructionResult_OkCopyWith(
          _$WalletInstructionResult_Ok value, $Res Function(_$WalletInstructionResult_Ok) then) =
      __$$WalletInstructionResult_OkCopyWithImpl<$Res>;
}

/// @nodoc
class __$$WalletInstructionResult_OkCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_Ok>
    implements _$$WalletInstructionResult_OkCopyWith<$Res> {
  __$$WalletInstructionResult_OkCopyWithImpl(
      _$WalletInstructionResult_Ok _value, $Res Function(_$WalletInstructionResult_Ok) _then)
      : super(_value, _then);
}

/// @nodoc

class _$WalletInstructionResult_Ok implements WalletInstructionResult_Ok {
  const _$WalletInstructionResult_Ok();

  @override
  String toString() {
    return 'WalletInstructionResult.ok()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$WalletInstructionResult_Ok);
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
  const factory WalletInstructionResult_Ok() = _$WalletInstructionResult_Ok;
}

/// @nodoc
abstract class _$$WalletInstructionResult_IncorrectPinCopyWith<$Res> {
  factory _$$WalletInstructionResult_IncorrectPinCopyWith(
          _$WalletInstructionResult_IncorrectPin value, $Res Function(_$WalletInstructionResult_IncorrectPin) then) =
      __$$WalletInstructionResult_IncorrectPinCopyWithImpl<$Res>;
  @useResult
  $Res call({int leftoverAttempts, bool isFinalAttempt});
}

/// @nodoc
class __$$WalletInstructionResult_IncorrectPinCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_IncorrectPin>
    implements _$$WalletInstructionResult_IncorrectPinCopyWith<$Res> {
  __$$WalletInstructionResult_IncorrectPinCopyWithImpl(
      _$WalletInstructionResult_IncorrectPin _value, $Res Function(_$WalletInstructionResult_IncorrectPin) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? leftoverAttempts = null,
    Object? isFinalAttempt = null,
  }) {
    return _then(_$WalletInstructionResult_IncorrectPin(
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

class _$WalletInstructionResult_IncorrectPin implements WalletInstructionResult_IncorrectPin {
  const _$WalletInstructionResult_IncorrectPin({required this.leftoverAttempts, required this.isFinalAttempt});

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
            other is _$WalletInstructionResult_IncorrectPin &&
            (identical(other.leftoverAttempts, leftoverAttempts) || other.leftoverAttempts == leftoverAttempts) &&
            (identical(other.isFinalAttempt, isFinalAttempt) || other.isFinalAttempt == isFinalAttempt));
  }

  @override
  int get hashCode => Object.hash(runtimeType, leftoverAttempts, isFinalAttempt);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletInstructionResult_IncorrectPinCopyWith<_$WalletInstructionResult_IncorrectPin> get copyWith =>
      __$$WalletInstructionResult_IncorrectPinCopyWithImpl<_$WalletInstructionResult_IncorrectPin>(this, _$identity);

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
      required final bool isFinalAttempt}) = _$WalletInstructionResult_IncorrectPin;

  int get leftoverAttempts;
  bool get isFinalAttempt;
  @JsonKey(ignore: true)
  _$$WalletInstructionResult_IncorrectPinCopyWith<_$WalletInstructionResult_IncorrectPin> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletInstructionResult_TimeoutCopyWith<$Res> {
  factory _$$WalletInstructionResult_TimeoutCopyWith(
          _$WalletInstructionResult_Timeout value, $Res Function(_$WalletInstructionResult_Timeout) then) =
      __$$WalletInstructionResult_TimeoutCopyWithImpl<$Res>;
  @useResult
  $Res call({int timeoutMillis});
}

/// @nodoc
class __$$WalletInstructionResult_TimeoutCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_Timeout>
    implements _$$WalletInstructionResult_TimeoutCopyWith<$Res> {
  __$$WalletInstructionResult_TimeoutCopyWithImpl(
      _$WalletInstructionResult_Timeout _value, $Res Function(_$WalletInstructionResult_Timeout) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timeoutMillis = null,
  }) {
    return _then(_$WalletInstructionResult_Timeout(
      timeoutMillis: null == timeoutMillis
          ? _value.timeoutMillis
          : timeoutMillis // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$WalletInstructionResult_Timeout implements WalletInstructionResult_Timeout {
  const _$WalletInstructionResult_Timeout({required this.timeoutMillis});

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
            other is _$WalletInstructionResult_Timeout &&
            (identical(other.timeoutMillis, timeoutMillis) || other.timeoutMillis == timeoutMillis));
  }

  @override
  int get hashCode => Object.hash(runtimeType, timeoutMillis);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletInstructionResult_TimeoutCopyWith<_$WalletInstructionResult_Timeout> get copyWith =>
      __$$WalletInstructionResult_TimeoutCopyWithImpl<_$WalletInstructionResult_Timeout>(this, _$identity);

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
  const factory WalletInstructionResult_Timeout({required final int timeoutMillis}) = _$WalletInstructionResult_Timeout;

  int get timeoutMillis;
  @JsonKey(ignore: true)
  _$$WalletInstructionResult_TimeoutCopyWith<_$WalletInstructionResult_Timeout> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletInstructionResult_BlockedCopyWith<$Res> {
  factory _$$WalletInstructionResult_BlockedCopyWith(
          _$WalletInstructionResult_Blocked value, $Res Function(_$WalletInstructionResult_Blocked) then) =
      __$$WalletInstructionResult_BlockedCopyWithImpl<$Res>;
}

/// @nodoc
class __$$WalletInstructionResult_BlockedCopyWithImpl<$Res>
    extends _$WalletInstructionResultCopyWithImpl<$Res, _$WalletInstructionResult_Blocked>
    implements _$$WalletInstructionResult_BlockedCopyWith<$Res> {
  __$$WalletInstructionResult_BlockedCopyWithImpl(
      _$WalletInstructionResult_Blocked _value, $Res Function(_$WalletInstructionResult_Blocked) _then)
      : super(_value, _then);
}

/// @nodoc

class _$WalletInstructionResult_Blocked implements WalletInstructionResult_Blocked {
  const _$WalletInstructionResult_Blocked();

  @override
  String toString() {
    return 'WalletInstructionResult.blocked()';
  }

  @override
  bool operator ==(dynamic other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is _$WalletInstructionResult_Blocked);
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
  const factory WalletInstructionResult_Blocked() = _$WalletInstructionResult_Blocked;
}
