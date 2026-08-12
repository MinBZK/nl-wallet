// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'organization.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Organization {

 String get id; String get legalName; String get displayName;@LocalizedTextConverter() LocalizedText? get type;@LocalizedTextConverter() LocalizedText? get description;@AppImageDataConverter() AppImageData? get logo; String? get webUri; String? get supportUri; String? get privacyPolicyUri; String get countryCode; String get organizationId;
/// Create a copy of Organization
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OrganizationCopyWith<Organization> get copyWith => _$OrganizationCopyWithImpl<Organization>(this as Organization, _$identity);

  /// Serializes this Organization to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Organization&&(identical(other.id, id) || other.id == id)&&(identical(other.legalName, legalName) || other.legalName == legalName)&&(identical(other.displayName, displayName) || other.displayName == displayName)&&const DeepCollectionEquality().equals(other.type, type)&&const DeepCollectionEquality().equals(other.description, description)&&(identical(other.logo, logo) || other.logo == logo)&&(identical(other.webUri, webUri) || other.webUri == webUri)&&(identical(other.supportUri, supportUri) || other.supportUri == supportUri)&&(identical(other.privacyPolicyUri, privacyPolicyUri) || other.privacyPolicyUri == privacyPolicyUri)&&(identical(other.countryCode, countryCode) || other.countryCode == countryCode)&&(identical(other.organizationId, organizationId) || other.organizationId == organizationId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,legalName,displayName,const DeepCollectionEquality().hash(type),const DeepCollectionEquality().hash(description),logo,webUri,supportUri,privacyPolicyUri,countryCode,organizationId);

@override
String toString() {
  return 'Organization(id: $id, legalName: $legalName, displayName: $displayName, type: $type, description: $description, logo: $logo, webUri: $webUri, supportUri: $supportUri, privacyPolicyUri: $privacyPolicyUri, countryCode: $countryCode, organizationId: $organizationId)';
}


}

/// @nodoc
abstract mixin class $OrganizationCopyWith<$Res>  {
  factory $OrganizationCopyWith(Organization value, $Res Function(Organization) _then) = _$OrganizationCopyWithImpl;
@useResult
$Res call({
 String id, String legalName, String displayName,@LocalizedTextConverter() LocalizedText? type,@LocalizedTextConverter() LocalizedText? description,@AppImageDataConverter() AppImageData? logo, String? webUri, String? supportUri, String? privacyPolicyUri, String countryCode, String organizationId
});




}
/// @nodoc
class _$OrganizationCopyWithImpl<$Res>
    implements $OrganizationCopyWith<$Res> {
  _$OrganizationCopyWithImpl(this._self, this._then);

  final Organization _self;
  final $Res Function(Organization) _then;

/// Create a copy of Organization
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? legalName = null,Object? displayName = null,Object? type = freezed,Object? description = freezed,Object? logo = freezed,Object? webUri = freezed,Object? supportUri = freezed,Object? privacyPolicyUri = freezed,Object? countryCode = null,Object? organizationId = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,legalName: null == legalName ? _self.legalName : legalName // ignore: cast_nullable_to_non_nullable
as String,displayName: null == displayName ? _self.displayName : displayName // ignore: cast_nullable_to_non_nullable
as String,type: freezed == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as LocalizedText?,description: freezed == description ? _self.description : description // ignore: cast_nullable_to_non_nullable
as LocalizedText?,logo: freezed == logo ? _self.logo : logo // ignore: cast_nullable_to_non_nullable
as AppImageData?,webUri: freezed == webUri ? _self.webUri : webUri // ignore: cast_nullable_to_non_nullable
as String?,supportUri: freezed == supportUri ? _self.supportUri : supportUri // ignore: cast_nullable_to_non_nullable
as String?,privacyPolicyUri: freezed == privacyPolicyUri ? _self.privacyPolicyUri : privacyPolicyUri // ignore: cast_nullable_to_non_nullable
as String?,countryCode: null == countryCode ? _self.countryCode : countryCode // ignore: cast_nullable_to_non_nullable
as String,organizationId: null == organizationId ? _self.organizationId : organizationId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [Organization].
extension OrganizationPatterns on Organization {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _Organization value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _Organization() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _Organization value)  $default,){
final _that = this;
switch (_that) {
case _Organization():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _Organization value)?  $default,){
final _that = this;
switch (_that) {
case _Organization() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String legalName,  String displayName, @LocalizedTextConverter()  LocalizedText? type, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData? logo,  String? webUri,  String? supportUri,  String? privacyPolicyUri,  String countryCode,  String organizationId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _Organization() when $default != null:
return $default(_that.id,_that.legalName,_that.displayName,_that.type,_that.description,_that.logo,_that.webUri,_that.supportUri,_that.privacyPolicyUri,_that.countryCode,_that.organizationId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String legalName,  String displayName, @LocalizedTextConverter()  LocalizedText? type, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData? logo,  String? webUri,  String? supportUri,  String? privacyPolicyUri,  String countryCode,  String organizationId)  $default,) {final _that = this;
switch (_that) {
case _Organization():
return $default(_that.id,_that.legalName,_that.displayName,_that.type,_that.description,_that.logo,_that.webUri,_that.supportUri,_that.privacyPolicyUri,_that.countryCode,_that.organizationId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String legalName,  String displayName, @LocalizedTextConverter()  LocalizedText? type, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData? logo,  String? webUri,  String? supportUri,  String? privacyPolicyUri,  String countryCode,  String organizationId)?  $default,) {final _that = this;
switch (_that) {
case _Organization() when $default != null:
return $default(_that.id,_that.legalName,_that.displayName,_that.type,_that.description,_that.logo,_that.webUri,_that.supportUri,_that.privacyPolicyUri,_that.countryCode,_that.organizationId);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _Organization implements Organization {
  const _Organization({required this.id, required this.legalName, required this.displayName, @LocalizedTextConverter() final  LocalizedText? type, @LocalizedTextConverter() final  LocalizedText? description, @AppImageDataConverter() this.logo, this.webUri, this.supportUri, this.privacyPolicyUri, required this.countryCode, required this.organizationId}): _type = type,_description = description;
  factory _Organization.fromJson(Map<String, dynamic> json) => _$OrganizationFromJson(json);

@override final  String id;
@override final  String legalName;
@override final  String displayName;
 final  LocalizedText? _type;
@override@LocalizedTextConverter() LocalizedText? get type {
  final value = _type;
  if (value == null) return null;
  if (_type is EqualUnmodifiableMapView) return _type;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(value);
}

 final  LocalizedText? _description;
@override@LocalizedTextConverter() LocalizedText? get description {
  final value = _description;
  if (value == null) return null;
  if (_description is EqualUnmodifiableMapView) return _description;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(value);
}

@override@AppImageDataConverter() final  AppImageData? logo;
@override final  String? webUri;
@override final  String? supportUri;
@override final  String? privacyPolicyUri;
@override final  String countryCode;
@override final  String organizationId;

/// Create a copy of Organization
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$OrganizationCopyWith<_Organization> get copyWith => __$OrganizationCopyWithImpl<_Organization>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$OrganizationToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _Organization&&(identical(other.id, id) || other.id == id)&&(identical(other.legalName, legalName) || other.legalName == legalName)&&(identical(other.displayName, displayName) || other.displayName == displayName)&&const DeepCollectionEquality().equals(other._type, _type)&&const DeepCollectionEquality().equals(other._description, _description)&&(identical(other.logo, logo) || other.logo == logo)&&(identical(other.webUri, webUri) || other.webUri == webUri)&&(identical(other.supportUri, supportUri) || other.supportUri == supportUri)&&(identical(other.privacyPolicyUri, privacyPolicyUri) || other.privacyPolicyUri == privacyPolicyUri)&&(identical(other.countryCode, countryCode) || other.countryCode == countryCode)&&(identical(other.organizationId, organizationId) || other.organizationId == organizationId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,legalName,displayName,const DeepCollectionEquality().hash(_type),const DeepCollectionEquality().hash(_description),logo,webUri,supportUri,privacyPolicyUri,countryCode,organizationId);

@override
String toString() {
  return 'Organization(id: $id, legalName: $legalName, displayName: $displayName, type: $type, description: $description, logo: $logo, webUri: $webUri, supportUri: $supportUri, privacyPolicyUri: $privacyPolicyUri, countryCode: $countryCode, organizationId: $organizationId)';
}


}

/// @nodoc
abstract mixin class _$OrganizationCopyWith<$Res> implements $OrganizationCopyWith<$Res> {
  factory _$OrganizationCopyWith(_Organization value, $Res Function(_Organization) _then) = __$OrganizationCopyWithImpl;
@override @useResult
$Res call({
 String id, String legalName, String displayName,@LocalizedTextConverter() LocalizedText? type,@LocalizedTextConverter() LocalizedText? description,@AppImageDataConverter() AppImageData? logo, String? webUri, String? supportUri, String? privacyPolicyUri, String countryCode, String organizationId
});




}
/// @nodoc
class __$OrganizationCopyWithImpl<$Res>
    implements _$OrganizationCopyWith<$Res> {
  __$OrganizationCopyWithImpl(this._self, this._then);

  final _Organization _self;
  final $Res Function(_Organization) _then;

/// Create a copy of Organization
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? legalName = null,Object? displayName = null,Object? type = freezed,Object? description = freezed,Object? logo = freezed,Object? webUri = freezed,Object? supportUri = freezed,Object? privacyPolicyUri = freezed,Object? countryCode = null,Object? organizationId = null,}) {
  return _then(_Organization(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,legalName: null == legalName ? _self.legalName : legalName // ignore: cast_nullable_to_non_nullable
as String,displayName: null == displayName ? _self.displayName : displayName // ignore: cast_nullable_to_non_nullable
as String,type: freezed == type ? _self._type : type // ignore: cast_nullable_to_non_nullable
as LocalizedText?,description: freezed == description ? _self._description : description // ignore: cast_nullable_to_non_nullable
as LocalizedText?,logo: freezed == logo ? _self.logo : logo // ignore: cast_nullable_to_non_nullable
as AppImageData?,webUri: freezed == webUri ? _self.webUri : webUri // ignore: cast_nullable_to_non_nullable
as String?,supportUri: freezed == supportUri ? _self.supportUri : supportUri // ignore: cast_nullable_to_non_nullable
as String?,privacyPolicyUri: freezed == privacyPolicyUri ? _self.privacyPolicyUri : privacyPolicyUri // ignore: cast_nullable_to_non_nullable
as String?,countryCode: null == countryCode ? _self.countryCode : countryCode // ignore: cast_nullable_to_non_nullable
as String,organizationId: null == organizationId ? _self.organizationId : organizationId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
