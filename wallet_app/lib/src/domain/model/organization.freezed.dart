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

 String get id;@LocalizedTextConverter() String get legalName;@LocalizedTextConverter() String get displayName;@LocalizedTextConverter() LocalizedText? get category;@LocalizedTextConverter() LocalizedText? get description;@AppImageDataConverter() AppImageData get logo; String? get webUrl; String? get privacyPolicyUrl; String get countryCode;@LocalizedTextConverter() LocalizedText? get city;@LocalizedTextConverter() LocalizedText? get department; String? get organizationId;
/// Create a copy of Organization
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OrganizationCopyWith<Organization> get copyWith => _$OrganizationCopyWithImpl<Organization>(this as Organization, _$identity);

  /// Serializes this Organization to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Organization&&(identical(other.id, id) || other.id == id)&&(identical(other.legalName, legalName) || other.legalName == legalName)&&(identical(other.displayName, displayName) || other.displayName == displayName)&&const DeepCollectionEquality().equals(other.category, category)&&const DeepCollectionEquality().equals(other.description, description)&&(identical(other.logo, logo) || other.logo == logo)&&(identical(other.webUrl, webUrl) || other.webUrl == webUrl)&&(identical(other.privacyPolicyUrl, privacyPolicyUrl) || other.privacyPolicyUrl == privacyPolicyUrl)&&(identical(other.countryCode, countryCode) || other.countryCode == countryCode)&&const DeepCollectionEquality().equals(other.city, city)&&const DeepCollectionEquality().equals(other.department, department)&&(identical(other.organizationId, organizationId) || other.organizationId == organizationId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,legalName,displayName,const DeepCollectionEquality().hash(category),const DeepCollectionEquality().hash(description),logo,webUrl,privacyPolicyUrl,countryCode,const DeepCollectionEquality().hash(city),const DeepCollectionEquality().hash(department),organizationId);

@override
String toString() {
  return 'Organization(id: $id, legalName: $legalName, displayName: $displayName, category: $category, description: $description, logo: $logo, webUrl: $webUrl, privacyPolicyUrl: $privacyPolicyUrl, countryCode: $countryCode, city: $city, department: $department, organizationId: $organizationId)';
}


}

/// @nodoc
abstract mixin class $OrganizationCopyWith<$Res>  {
  factory $OrganizationCopyWith(Organization value, $Res Function(Organization) _then) = _$OrganizationCopyWithImpl;
@useResult
$Res call({
 String id,@LocalizedTextConverter() String legalName,@LocalizedTextConverter() String displayName,@LocalizedTextConverter() LocalizedText? category,@LocalizedTextConverter() LocalizedText? description,@AppImageDataConverter() AppImageData logo, String? webUrl, String? privacyPolicyUrl, String countryCode,@LocalizedTextConverter() LocalizedText? city,@LocalizedTextConverter() LocalizedText? department, String? organizationId
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
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? legalName = null,Object? displayName = null,Object? category = freezed,Object? description = freezed,Object? logo = null,Object? webUrl = freezed,Object? privacyPolicyUrl = freezed,Object? countryCode = null,Object? city = freezed,Object? department = freezed,Object? organizationId = freezed,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,legalName: null == legalName ? _self.legalName : legalName // ignore: cast_nullable_to_non_nullable
as String,displayName: null == displayName ? _self.displayName : displayName // ignore: cast_nullable_to_non_nullable
as String,category: freezed == category ? _self.category : category // ignore: cast_nullable_to_non_nullable
as LocalizedText?,description: freezed == description ? _self.description : description // ignore: cast_nullable_to_non_nullable
as LocalizedText?,logo: null == logo ? _self.logo : logo // ignore: cast_nullable_to_non_nullable
as AppImageData,webUrl: freezed == webUrl ? _self.webUrl : webUrl // ignore: cast_nullable_to_non_nullable
as String?,privacyPolicyUrl: freezed == privacyPolicyUrl ? _self.privacyPolicyUrl : privacyPolicyUrl // ignore: cast_nullable_to_non_nullable
as String?,countryCode: null == countryCode ? _self.countryCode : countryCode // ignore: cast_nullable_to_non_nullable
as String,city: freezed == city ? _self.city : city // ignore: cast_nullable_to_non_nullable
as LocalizedText?,department: freezed == department ? _self.department : department // ignore: cast_nullable_to_non_nullable
as LocalizedText?,organizationId: freezed == organizationId ? _self.organizationId : organizationId // ignore: cast_nullable_to_non_nullable
as String?,
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id, @LocalizedTextConverter()  String legalName, @LocalizedTextConverter()  String displayName, @LocalizedTextConverter()  LocalizedText? category, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData logo,  String? webUrl,  String? privacyPolicyUrl,  String countryCode, @LocalizedTextConverter()  LocalizedText? city, @LocalizedTextConverter()  LocalizedText? department,  String? organizationId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _Organization() when $default != null:
return $default(_that.id,_that.legalName,_that.displayName,_that.category,_that.description,_that.logo,_that.webUrl,_that.privacyPolicyUrl,_that.countryCode,_that.city,_that.department,_that.organizationId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id, @LocalizedTextConverter()  String legalName, @LocalizedTextConverter()  String displayName, @LocalizedTextConverter()  LocalizedText? category, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData logo,  String? webUrl,  String? privacyPolicyUrl,  String countryCode, @LocalizedTextConverter()  LocalizedText? city, @LocalizedTextConverter()  LocalizedText? department,  String? organizationId)  $default,) {final _that = this;
switch (_that) {
case _Organization():
return $default(_that.id,_that.legalName,_that.displayName,_that.category,_that.description,_that.logo,_that.webUrl,_that.privacyPolicyUrl,_that.countryCode,_that.city,_that.department,_that.organizationId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id, @LocalizedTextConverter()  String legalName, @LocalizedTextConverter()  String displayName, @LocalizedTextConverter()  LocalizedText? category, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData logo,  String? webUrl,  String? privacyPolicyUrl,  String countryCode, @LocalizedTextConverter()  LocalizedText? city, @LocalizedTextConverter()  LocalizedText? department,  String? organizationId)?  $default,) {final _that = this;
switch (_that) {
case _Organization() when $default != null:
return $default(_that.id,_that.legalName,_that.displayName,_that.category,_that.description,_that.logo,_that.webUrl,_that.privacyPolicyUrl,_that.countryCode,_that.city,_that.department,_that.organizationId);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _Organization implements Organization {
  const _Organization({required this.id, @LocalizedTextConverter() required this.legalName, @LocalizedTextConverter() required this.displayName, @LocalizedTextConverter() required final  LocalizedText? category, @LocalizedTextConverter() required final  LocalizedText? description, @AppImageDataConverter() required this.logo, this.webUrl, this.privacyPolicyUrl, required this.countryCode, @LocalizedTextConverter() final  LocalizedText? city, @LocalizedTextConverter() final  LocalizedText? department, this.organizationId}): _category = category,_description = description,_city = city,_department = department;
  factory _Organization.fromJson(Map<String, dynamic> json) => _$OrganizationFromJson(json);

@override final  String id;
@override@LocalizedTextConverter() final  String legalName;
@override@LocalizedTextConverter() final  String displayName;
 final  LocalizedText? _category;
@override@LocalizedTextConverter() LocalizedText? get category {
  final value = _category;
  if (value == null) return null;
  if (_category is EqualUnmodifiableMapView) return _category;
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

@override@AppImageDataConverter() final  AppImageData logo;
@override final  String? webUrl;
@override final  String? privacyPolicyUrl;
@override final  String countryCode;
 final  LocalizedText? _city;
@override@LocalizedTextConverter() LocalizedText? get city {
  final value = _city;
  if (value == null) return null;
  if (_city is EqualUnmodifiableMapView) return _city;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(value);
}

 final  LocalizedText? _department;
@override@LocalizedTextConverter() LocalizedText? get department {
  final value = _department;
  if (value == null) return null;
  if (_department is EqualUnmodifiableMapView) return _department;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(value);
}

@override final  String? organizationId;

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
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _Organization&&(identical(other.id, id) || other.id == id)&&(identical(other.legalName, legalName) || other.legalName == legalName)&&(identical(other.displayName, displayName) || other.displayName == displayName)&&const DeepCollectionEquality().equals(other._category, _category)&&const DeepCollectionEquality().equals(other._description, _description)&&(identical(other.logo, logo) || other.logo == logo)&&(identical(other.webUrl, webUrl) || other.webUrl == webUrl)&&(identical(other.privacyPolicyUrl, privacyPolicyUrl) || other.privacyPolicyUrl == privacyPolicyUrl)&&(identical(other.countryCode, countryCode) || other.countryCode == countryCode)&&const DeepCollectionEquality().equals(other._city, _city)&&const DeepCollectionEquality().equals(other._department, _department)&&(identical(other.organizationId, organizationId) || other.organizationId == organizationId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,legalName,displayName,const DeepCollectionEquality().hash(_category),const DeepCollectionEquality().hash(_description),logo,webUrl,privacyPolicyUrl,countryCode,const DeepCollectionEquality().hash(_city),const DeepCollectionEquality().hash(_department),organizationId);

@override
String toString() {
  return 'Organization(id: $id, legalName: $legalName, displayName: $displayName, category: $category, description: $description, logo: $logo, webUrl: $webUrl, privacyPolicyUrl: $privacyPolicyUrl, countryCode: $countryCode, city: $city, department: $department, organizationId: $organizationId)';
}


}

/// @nodoc
abstract mixin class _$OrganizationCopyWith<$Res> implements $OrganizationCopyWith<$Res> {
  factory _$OrganizationCopyWith(_Organization value, $Res Function(_Organization) _then) = __$OrganizationCopyWithImpl;
@override @useResult
$Res call({
 String id,@LocalizedTextConverter() String legalName,@LocalizedTextConverter() String displayName,@LocalizedTextConverter() LocalizedText? category,@LocalizedTextConverter() LocalizedText? description,@AppImageDataConverter() AppImageData logo, String? webUrl, String? privacyPolicyUrl, String countryCode,@LocalizedTextConverter() LocalizedText? city,@LocalizedTextConverter() LocalizedText? department, String? organizationId
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
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? legalName = null,Object? displayName = null,Object? category = freezed,Object? description = freezed,Object? logo = null,Object? webUrl = freezed,Object? privacyPolicyUrl = freezed,Object? countryCode = null,Object? city = freezed,Object? department = freezed,Object? organizationId = freezed,}) {
  return _then(_Organization(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,legalName: null == legalName ? _self.legalName : legalName // ignore: cast_nullable_to_non_nullable
as String,displayName: null == displayName ? _self.displayName : displayName // ignore: cast_nullable_to_non_nullable
as String,category: freezed == category ? _self._category : category // ignore: cast_nullable_to_non_nullable
as LocalizedText?,description: freezed == description ? _self._description : description // ignore: cast_nullable_to_non_nullable
as LocalizedText?,logo: null == logo ? _self.logo : logo // ignore: cast_nullable_to_non_nullable
as AppImageData,webUrl: freezed == webUrl ? _self.webUrl : webUrl // ignore: cast_nullable_to_non_nullable
as String?,privacyPolicyUrl: freezed == privacyPolicyUrl ? _self.privacyPolicyUrl : privacyPolicyUrl // ignore: cast_nullable_to_non_nullable
as String?,countryCode: null == countryCode ? _self.countryCode : countryCode // ignore: cast_nullable_to_non_nullable
as String,city: freezed == city ? _self._city : city // ignore: cast_nullable_to_non_nullable
as LocalizedText?,department: freezed == department ? _self._department : department // ignore: cast_nullable_to_non_nullable
as LocalizedText?,organizationId: freezed == organizationId ? _self.organizationId : organizationId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
