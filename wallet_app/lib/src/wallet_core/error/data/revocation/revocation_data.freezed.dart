// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'revocation_data.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$RevocationData {

@JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown) RevocationReason get revocationReason;@JsonKey(name: 'can_register_new_account') bool get canRegisterNewAccount;
/// Create a copy of RevocationData
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RevocationDataCopyWith<RevocationData> get copyWith => _$RevocationDataCopyWithImpl<RevocationData>(this as RevocationData, _$identity);

  /// Serializes this RevocationData to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RevocationData&&(identical(other.revocationReason, revocationReason) || other.revocationReason == revocationReason)&&(identical(other.canRegisterNewAccount, canRegisterNewAccount) || other.canRegisterNewAccount == canRegisterNewAccount));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,revocationReason,canRegisterNewAccount);

@override
String toString() {
  return 'RevocationData(revocationReason: $revocationReason, canRegisterNewAccount: $canRegisterNewAccount)';
}


}

/// @nodoc
abstract mixin class $RevocationDataCopyWith<$Res>  {
  factory $RevocationDataCopyWith(RevocationData value, $Res Function(RevocationData) _then) = _$RevocationDataCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown) RevocationReason revocationReason,@JsonKey(name: 'can_register_new_account') bool canRegisterNewAccount
});




}
/// @nodoc
class _$RevocationDataCopyWithImpl<$Res>
    implements $RevocationDataCopyWith<$Res> {
  _$RevocationDataCopyWithImpl(this._self, this._then);

  final RevocationData _self;
  final $Res Function(RevocationData) _then;

/// Create a copy of RevocationData
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? revocationReason = null,Object? canRegisterNewAccount = null,}) {
  return _then(_self.copyWith(
revocationReason: null == revocationReason ? _self.revocationReason : revocationReason // ignore: cast_nullable_to_non_nullable
as RevocationReason,canRegisterNewAccount: null == canRegisterNewAccount ? _self.canRegisterNewAccount : canRegisterNewAccount // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [RevocationData].
extension RevocationDataPatterns on RevocationData {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _RevocationData value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _RevocationData() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _RevocationData value)  $default,){
final _that = this;
switch (_that) {
case _RevocationData():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _RevocationData value)?  $default,){
final _that = this;
switch (_that) {
case _RevocationData() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown)  RevocationReason revocationReason, @JsonKey(name: 'can_register_new_account')  bool canRegisterNewAccount)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _RevocationData() when $default != null:
return $default(_that.revocationReason,_that.canRegisterNewAccount);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown)  RevocationReason revocationReason, @JsonKey(name: 'can_register_new_account')  bool canRegisterNewAccount)  $default,) {final _that = this;
switch (_that) {
case _RevocationData():
return $default(_that.revocationReason,_that.canRegisterNewAccount);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown)  RevocationReason revocationReason, @JsonKey(name: 'can_register_new_account')  bool canRegisterNewAccount)?  $default,) {final _that = this;
switch (_that) {
case _RevocationData() when $default != null:
return $default(_that.revocationReason,_that.canRegisterNewAccount);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _RevocationData implements RevocationData {
   _RevocationData({@JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown) required this.revocationReason, @JsonKey(name: 'can_register_new_account') required this.canRegisterNewAccount});
  factory _RevocationData.fromJson(Map<String, dynamic> json) => _$RevocationDataFromJson(json);

@override@JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown) final  RevocationReason revocationReason;
@override@JsonKey(name: 'can_register_new_account') final  bool canRegisterNewAccount;

/// Create a copy of RevocationData
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$RevocationDataCopyWith<_RevocationData> get copyWith => __$RevocationDataCopyWithImpl<_RevocationData>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$RevocationDataToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _RevocationData&&(identical(other.revocationReason, revocationReason) || other.revocationReason == revocationReason)&&(identical(other.canRegisterNewAccount, canRegisterNewAccount) || other.canRegisterNewAccount == canRegisterNewAccount));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,revocationReason,canRegisterNewAccount);

@override
String toString() {
  return 'RevocationData(revocationReason: $revocationReason, canRegisterNewAccount: $canRegisterNewAccount)';
}


}

/// @nodoc
abstract mixin class _$RevocationDataCopyWith<$Res> implements $RevocationDataCopyWith<$Res> {
  factory _$RevocationDataCopyWith(_RevocationData value, $Res Function(_RevocationData) _then) = __$RevocationDataCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown) RevocationReason revocationReason,@JsonKey(name: 'can_register_new_account') bool canRegisterNewAccount
});




}
/// @nodoc
class __$RevocationDataCopyWithImpl<$Res>
    implements _$RevocationDataCopyWith<$Res> {
  __$RevocationDataCopyWithImpl(this._self, this._then);

  final _RevocationData _self;
  final $Res Function(_RevocationData) _then;

/// Create a copy of RevocationData
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? revocationReason = null,Object? canRegisterNewAccount = null,}) {
  return _then(_RevocationData(
revocationReason: null == revocationReason ? _self.revocationReason : revocationReason // ignore: cast_nullable_to_non_nullable
as RevocationReason,canRegisterNewAccount: null == canRegisterNewAccount ? _self.canRegisterNewAccount : canRegisterNewAccount // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
