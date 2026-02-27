// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'core_error_data.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$CoreErrorData {

@JsonKey(name: 'revocation_data') RevocationData? get revocationData;@JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown) RedirectError? get redirectError;@JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown) SessionType? get sessionType;@JsonKey(name: 'can_retry') bool? get canRetry;@JsonKey(name: 'organization_name') Map<String, dynamic>? get organizationName;
/// Create a copy of CoreErrorData
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CoreErrorDataCopyWith<CoreErrorData> get copyWith => _$CoreErrorDataCopyWithImpl<CoreErrorData>(this as CoreErrorData, _$identity);

  /// Serializes this CoreErrorData to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CoreErrorData&&(identical(other.revocationData, revocationData) || other.revocationData == revocationData)&&(identical(other.redirectError, redirectError) || other.redirectError == redirectError)&&(identical(other.sessionType, sessionType) || other.sessionType == sessionType)&&(identical(other.canRetry, canRetry) || other.canRetry == canRetry)&&const DeepCollectionEquality().equals(other.organizationName, organizationName));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,revocationData,redirectError,sessionType,canRetry,const DeepCollectionEquality().hash(organizationName));

@override
String toString() {
  return 'CoreErrorData(revocationData: $revocationData, redirectError: $redirectError, sessionType: $sessionType, canRetry: $canRetry, organizationName: $organizationName)';
}


}

/// @nodoc
abstract mixin class $CoreErrorDataCopyWith<$Res>  {
  factory $CoreErrorDataCopyWith(CoreErrorData value, $Res Function(CoreErrorData) _then) = _$CoreErrorDataCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'revocation_data') RevocationData? revocationData,@JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown) RedirectError? redirectError,@JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown) SessionType? sessionType,@JsonKey(name: 'can_retry') bool? canRetry,@JsonKey(name: 'organization_name') Map<String, dynamic>? organizationName
});


$RevocationDataCopyWith<$Res>? get revocationData;

}
/// @nodoc
class _$CoreErrorDataCopyWithImpl<$Res>
    implements $CoreErrorDataCopyWith<$Res> {
  _$CoreErrorDataCopyWithImpl(this._self, this._then);

  final CoreErrorData _self;
  final $Res Function(CoreErrorData) _then;

/// Create a copy of CoreErrorData
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? revocationData = freezed,Object? redirectError = freezed,Object? sessionType = freezed,Object? canRetry = freezed,Object? organizationName = freezed,}) {
  return _then(_self.copyWith(
revocationData: freezed == revocationData ? _self.revocationData : revocationData // ignore: cast_nullable_to_non_nullable
as RevocationData?,redirectError: freezed == redirectError ? _self.redirectError : redirectError // ignore: cast_nullable_to_non_nullable
as RedirectError?,sessionType: freezed == sessionType ? _self.sessionType : sessionType // ignore: cast_nullable_to_non_nullable
as SessionType?,canRetry: freezed == canRetry ? _self.canRetry : canRetry // ignore: cast_nullable_to_non_nullable
as bool?,organizationName: freezed == organizationName ? _self.organizationName : organizationName // ignore: cast_nullable_to_non_nullable
as Map<String, dynamic>?,
  ));
}
/// Create a copy of CoreErrorData
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$RevocationDataCopyWith<$Res>? get revocationData {
    if (_self.revocationData == null) {
    return null;
  }

  return $RevocationDataCopyWith<$Res>(_self.revocationData!, (value) {
    return _then(_self.copyWith(revocationData: value));
  });
}
}


/// Adds pattern-matching-related methods to [CoreErrorData].
extension CoreErrorDataPatterns on CoreErrorData {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CoreErrorData value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CoreErrorData() when $default != null:
return $default(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CoreErrorData value)  $default,){
final _that = this;
switch (_that) {
case _CoreErrorData():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CoreErrorData value)?  $default,){
final _that = this;
switch (_that) {
case _CoreErrorData() when $default != null:
return $default(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(name: 'revocation_data')  RevocationData? revocationData, @JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown)  RedirectError? redirectError, @JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown)  SessionType? sessionType, @JsonKey(name: 'can_retry')  bool? canRetry, @JsonKey(name: 'organization_name')  Map<String, dynamic>? organizationName)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CoreErrorData() when $default != null:
return $default(_that.revocationData,_that.redirectError,_that.sessionType,_that.canRetry,_that.organizationName);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(name: 'revocation_data')  RevocationData? revocationData, @JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown)  RedirectError? redirectError, @JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown)  SessionType? sessionType, @JsonKey(name: 'can_retry')  bool? canRetry, @JsonKey(name: 'organization_name')  Map<String, dynamic>? organizationName)  $default,) {final _that = this;
switch (_that) {
case _CoreErrorData():
return $default(_that.revocationData,_that.redirectError,_that.sessionType,_that.canRetry,_that.organizationName);case _:
  throw StateError('Unexpected subclass');

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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(name: 'revocation_data')  RevocationData? revocationData, @JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown)  RedirectError? redirectError, @JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown)  SessionType? sessionType, @JsonKey(name: 'can_retry')  bool? canRetry, @JsonKey(name: 'organization_name')  Map<String, dynamic>? organizationName)?  $default,) {final _that = this;
switch (_that) {
case _CoreErrorData() when $default != null:
return $default(_that.revocationData,_that.redirectError,_that.sessionType,_that.canRetry,_that.organizationName);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _CoreErrorData extends CoreErrorData {
   _CoreErrorData({@JsonKey(name: 'revocation_data') this.revocationData, @JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown) this.redirectError, @JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown) this.sessionType, @JsonKey(name: 'can_retry') this.canRetry, @JsonKey(name: 'organization_name') final  Map<String, dynamic>? organizationName}): _organizationName = organizationName,super._();
  factory _CoreErrorData.fromJson(Map<String, dynamic> json) => _$CoreErrorDataFromJson(json);

@override@JsonKey(name: 'revocation_data') final  RevocationData? revocationData;
@override@JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown) final  RedirectError? redirectError;
@override@JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown) final  SessionType? sessionType;
@override@JsonKey(name: 'can_retry') final  bool? canRetry;
 final  Map<String, dynamic>? _organizationName;
@override@JsonKey(name: 'organization_name') Map<String, dynamic>? get organizationName {
  final value = _organizationName;
  if (value == null) return null;
  if (_organizationName is EqualUnmodifiableMapView) return _organizationName;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(value);
}


/// Create a copy of CoreErrorData
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CoreErrorDataCopyWith<_CoreErrorData> get copyWith => __$CoreErrorDataCopyWithImpl<_CoreErrorData>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$CoreErrorDataToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CoreErrorData&&(identical(other.revocationData, revocationData) || other.revocationData == revocationData)&&(identical(other.redirectError, redirectError) || other.redirectError == redirectError)&&(identical(other.sessionType, sessionType) || other.sessionType == sessionType)&&(identical(other.canRetry, canRetry) || other.canRetry == canRetry)&&const DeepCollectionEquality().equals(other._organizationName, _organizationName));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,revocationData,redirectError,sessionType,canRetry,const DeepCollectionEquality().hash(_organizationName));

@override
String toString() {
  return 'CoreErrorData(revocationData: $revocationData, redirectError: $redirectError, sessionType: $sessionType, canRetry: $canRetry, organizationName: $organizationName)';
}


}

/// @nodoc
abstract mixin class _$CoreErrorDataCopyWith<$Res> implements $CoreErrorDataCopyWith<$Res> {
  factory _$CoreErrorDataCopyWith(_CoreErrorData value, $Res Function(_CoreErrorData) _then) = __$CoreErrorDataCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(name: 'revocation_data') RevocationData? revocationData,@JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown) RedirectError? redirectError,@JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown) SessionType? sessionType,@JsonKey(name: 'can_retry') bool? canRetry,@JsonKey(name: 'organization_name') Map<String, dynamic>? organizationName
});


@override $RevocationDataCopyWith<$Res>? get revocationData;

}
/// @nodoc
class __$CoreErrorDataCopyWithImpl<$Res>
    implements _$CoreErrorDataCopyWith<$Res> {
  __$CoreErrorDataCopyWithImpl(this._self, this._then);

  final _CoreErrorData _self;
  final $Res Function(_CoreErrorData) _then;

/// Create a copy of CoreErrorData
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? revocationData = freezed,Object? redirectError = freezed,Object? sessionType = freezed,Object? canRetry = freezed,Object? organizationName = freezed,}) {
  return _then(_CoreErrorData(
revocationData: freezed == revocationData ? _self.revocationData : revocationData // ignore: cast_nullable_to_non_nullable
as RevocationData?,redirectError: freezed == redirectError ? _self.redirectError : redirectError // ignore: cast_nullable_to_non_nullable
as RedirectError?,sessionType: freezed == sessionType ? _self.sessionType : sessionType // ignore: cast_nullable_to_non_nullable
as SessionType?,canRetry: freezed == canRetry ? _self.canRetry : canRetry // ignore: cast_nullable_to_non_nullable
as bool?,organizationName: freezed == organizationName ? _self._organizationName : organizationName // ignore: cast_nullable_to_non_nullable
as Map<String, dynamic>?,
  ));
}

/// Create a copy of CoreErrorData
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$RevocationDataCopyWith<$Res>? get revocationData {
    if (_self.revocationData == null) {
    return null;
  }

  return $RevocationDataCopyWith<$Res>(_self.revocationData!, (value) {
    return _then(_self.copyWith(revocationData: value));
  });
}
}

// dart format on
