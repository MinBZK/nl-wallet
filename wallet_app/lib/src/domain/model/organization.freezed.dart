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

 String get id;@LocalizedTextConverter() LocalizedText get legalName;@LocalizedTextConverter() LocalizedText get displayName;@LocalizedTextConverter() LocalizedText? get category;@LocalizedTextConverter() LocalizedText? get description;@AppImageDataConverter() AppImageData get logo; String? get webUrl; String? get privacyPolicyUrl; String? get countryCode;@LocalizedTextConverter() LocalizedText? get city;@LocalizedTextConverter() LocalizedText? get department; String? get kvk;
/// Create a copy of Organization
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OrganizationCopyWith<Organization> get copyWith => _$OrganizationCopyWithImpl<Organization>(this as Organization, _$identity);

  /// Serializes this Organization to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Organization&&(identical(other.id, id) || other.id == id)&&const DeepCollectionEquality().equals(other.legalName, legalName)&&const DeepCollectionEquality().equals(other.displayName, displayName)&&const DeepCollectionEquality().equals(other.category, category)&&const DeepCollectionEquality().equals(other.description, description)&&(identical(other.logo, logo) || other.logo == logo)&&(identical(other.webUrl, webUrl) || other.webUrl == webUrl)&&(identical(other.privacyPolicyUrl, privacyPolicyUrl) || other.privacyPolicyUrl == privacyPolicyUrl)&&(identical(other.countryCode, countryCode) || other.countryCode == countryCode)&&const DeepCollectionEquality().equals(other.city, city)&&const DeepCollectionEquality().equals(other.department, department)&&(identical(other.kvk, kvk) || other.kvk == kvk));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,const DeepCollectionEquality().hash(legalName),const DeepCollectionEquality().hash(displayName),const DeepCollectionEquality().hash(category),const DeepCollectionEquality().hash(description),logo,webUrl,privacyPolicyUrl,countryCode,const DeepCollectionEquality().hash(city),const DeepCollectionEquality().hash(department),kvk);

@override
String toString() {
  return 'Organization(id: $id, legalName: $legalName, displayName: $displayName, category: $category, description: $description, logo: $logo, webUrl: $webUrl, privacyPolicyUrl: $privacyPolicyUrl, countryCode: $countryCode, city: $city, department: $department, kvk: $kvk)';
}


}

/// @nodoc
abstract mixin class $OrganizationCopyWith<$Res>  {
  factory $OrganizationCopyWith(Organization value, $Res Function(Organization) _then) = _$OrganizationCopyWithImpl;
@useResult
$Res call({
 String id,@LocalizedTextConverter() LocalizedText legalName,@LocalizedTextConverter() LocalizedText displayName,@LocalizedTextConverter() LocalizedText? category,@LocalizedTextConverter() LocalizedText? description,@AppImageDataConverter() AppImageData logo, String? webUrl, String? privacyPolicyUrl, String? countryCode,@LocalizedTextConverter() LocalizedText? city,@LocalizedTextConverter() LocalizedText? department, String? kvk
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
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? legalName = null,Object? displayName = null,Object? category = freezed,Object? description = freezed,Object? logo = null,Object? webUrl = freezed,Object? privacyPolicyUrl = freezed,Object? countryCode = freezed,Object? city = freezed,Object? department = freezed,Object? kvk = freezed,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,legalName: null == legalName ? _self.legalName : legalName // ignore: cast_nullable_to_non_nullable
as LocalizedText,displayName: null == displayName ? _self.displayName : displayName // ignore: cast_nullable_to_non_nullable
as LocalizedText,category: freezed == category ? _self.category : category // ignore: cast_nullable_to_non_nullable
as LocalizedText?,description: freezed == description ? _self.description : description // ignore: cast_nullable_to_non_nullable
as LocalizedText?,logo: null == logo ? _self.logo : logo // ignore: cast_nullable_to_non_nullable
as AppImageData,webUrl: freezed == webUrl ? _self.webUrl : webUrl // ignore: cast_nullable_to_non_nullable
as String?,privacyPolicyUrl: freezed == privacyPolicyUrl ? _self.privacyPolicyUrl : privacyPolicyUrl // ignore: cast_nullable_to_non_nullable
as String?,countryCode: freezed == countryCode ? _self.countryCode : countryCode // ignore: cast_nullable_to_non_nullable
as String?,city: freezed == city ? _self.city : city // ignore: cast_nullable_to_non_nullable
as LocalizedText?,department: freezed == department ? _self.department : department // ignore: cast_nullable_to_non_nullable
as LocalizedText?,kvk: freezed == kvk ? _self.kvk : kvk // ignore: cast_nullable_to_non_nullable
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id, @LocalizedTextConverter()  LocalizedText legalName, @LocalizedTextConverter()  LocalizedText displayName, @LocalizedTextConverter()  LocalizedText? category, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData logo,  String? webUrl,  String? privacyPolicyUrl,  String? countryCode, @LocalizedTextConverter()  LocalizedText? city, @LocalizedTextConverter()  LocalizedText? department,  String? kvk)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _Organization() when $default != null:
return $default(_that.id,_that.legalName,_that.displayName,_that.category,_that.description,_that.logo,_that.webUrl,_that.privacyPolicyUrl,_that.countryCode,_that.city,_that.department,_that.kvk);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id, @LocalizedTextConverter()  LocalizedText legalName, @LocalizedTextConverter()  LocalizedText displayName, @LocalizedTextConverter()  LocalizedText? category, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData logo,  String? webUrl,  String? privacyPolicyUrl,  String? countryCode, @LocalizedTextConverter()  LocalizedText? city, @LocalizedTextConverter()  LocalizedText? department,  String? kvk)  $default,) {final _that = this;
switch (_that) {
case _Organization():
return $default(_that.id,_that.legalName,_that.displayName,_that.category,_that.description,_that.logo,_that.webUrl,_that.privacyPolicyUrl,_that.countryCode,_that.city,_that.department,_that.kvk);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id, @LocalizedTextConverter()  LocalizedText legalName, @LocalizedTextConverter()  LocalizedText displayName, @LocalizedTextConverter()  LocalizedText? category, @LocalizedTextConverter()  LocalizedText? description, @AppImageDataConverter()  AppImageData logo,  String? webUrl,  String? privacyPolicyUrl,  String? countryCode, @LocalizedTextConverter()  LocalizedText? city, @LocalizedTextConverter()  LocalizedText? department,  String? kvk)?  $default,) {final _that = this;
switch (_that) {
case _Organization() when $default != null:
return $default(_that.id,_that.legalName,_that.displayName,_that.category,_that.description,_that.logo,_that.webUrl,_that.privacyPolicyUrl,_that.countryCode,_that.city,_that.department,_that.kvk);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _Organization implements Organization {
  const _Organization({required this.id, @LocalizedTextConverter() required final  LocalizedText legalName, @LocalizedTextConverter() required final  LocalizedText displayName, @LocalizedTextConverter() required final  LocalizedText? category, @LocalizedTextConverter() required final  LocalizedText? description, @AppImageDataConverter() required this.logo, this.webUrl, this.privacyPolicyUrl, this.countryCode, @LocalizedTextConverter() final  LocalizedText? city, @LocalizedTextConverter() final  LocalizedText? department, this.kvk}): _legalName = legalName,_displayName = displayName,_category = category,_description = description,_city = city,_department = department;
  factory _Organization.fromJson(Map<String, dynamic> json) => _$OrganizationFromJson(json);

@override final  String id;
 final  LocalizedText _legalName;
@override@LocalizedTextConverter() LocalizedText get legalName {
  if (_legalName is EqualUnmodifiableMapView) return _legalName;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_legalName);
}

 final  LocalizedText _displayName;
@override@LocalizedTextConverter() LocalizedText get displayName {
  if (_displayName is EqualUnmodifiableMapView) return _displayName;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_displayName);
}

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
@override final  String? countryCode;
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

@override final  String? kvk;

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
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _Organization&&(identical(other.id, id) || other.id == id)&&const DeepCollectionEquality().equals(other._legalName, _legalName)&&const DeepCollectionEquality().equals(other._displayName, _displayName)&&const DeepCollectionEquality().equals(other._category, _category)&&const DeepCollectionEquality().equals(other._description, _description)&&(identical(other.logo, logo) || other.logo == logo)&&(identical(other.webUrl, webUrl) || other.webUrl == webUrl)&&(identical(other.privacyPolicyUrl, privacyPolicyUrl) || other.privacyPolicyUrl == privacyPolicyUrl)&&(identical(other.countryCode, countryCode) || other.countryCode == countryCode)&&const DeepCollectionEquality().equals(other._city, _city)&&const DeepCollectionEquality().equals(other._department, _department)&&(identical(other.kvk, kvk) || other.kvk == kvk));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,const DeepCollectionEquality().hash(_legalName),const DeepCollectionEquality().hash(_displayName),const DeepCollectionEquality().hash(_category),const DeepCollectionEquality().hash(_description),logo,webUrl,privacyPolicyUrl,countryCode,const DeepCollectionEquality().hash(_city),const DeepCollectionEquality().hash(_department),kvk);

@override
String toString() {
  return 'Organization(id: $id, legalName: $legalName, displayName: $displayName, category: $category, description: $description, logo: $logo, webUrl: $webUrl, privacyPolicyUrl: $privacyPolicyUrl, countryCode: $countryCode, city: $city, department: $department, kvk: $kvk)';
}


}

/// @nodoc
abstract mixin class _$OrganizationCopyWith<$Res> implements $OrganizationCopyWith<$Res> {
  factory _$OrganizationCopyWith(_Organization value, $Res Function(_Organization) _then) = __$OrganizationCopyWithImpl;
@override @useResult
$Res call({
 String id,@LocalizedTextConverter() LocalizedText legalName,@LocalizedTextConverter() LocalizedText displayName,@LocalizedTextConverter() LocalizedText? category,@LocalizedTextConverter() LocalizedText? description,@AppImageDataConverter() AppImageData logo, String? webUrl, String? privacyPolicyUrl, String? countryCode,@LocalizedTextConverter() LocalizedText? city,@LocalizedTextConverter() LocalizedText? department, String? kvk
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
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? legalName = null,Object? displayName = null,Object? category = freezed,Object? description = freezed,Object? logo = null,Object? webUrl = freezed,Object? privacyPolicyUrl = freezed,Object? countryCode = freezed,Object? city = freezed,Object? department = freezed,Object? kvk = freezed,}) {
  return _then(_Organization(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,legalName: null == legalName ? _self._legalName : legalName // ignore: cast_nullable_to_non_nullable
as LocalizedText,displayName: null == displayName ? _self._displayName : displayName // ignore: cast_nullable_to_non_nullable
as LocalizedText,category: freezed == category ? _self._category : category // ignore: cast_nullable_to_non_nullable
as LocalizedText?,description: freezed == description ? _self._description : description // ignore: cast_nullable_to_non_nullable
as LocalizedText?,logo: null == logo ? _self.logo : logo // ignore: cast_nullable_to_non_nullable
as AppImageData,webUrl: freezed == webUrl ? _self.webUrl : webUrl // ignore: cast_nullable_to_non_nullable
as String?,privacyPolicyUrl: freezed == privacyPolicyUrl ? _self.privacyPolicyUrl : privacyPolicyUrl // ignore: cast_nullable_to_non_nullable
as String?,countryCode: freezed == countryCode ? _self.countryCode : countryCode // ignore: cast_nullable_to_non_nullable
as String?,city: freezed == city ? _self._city : city // ignore: cast_nullable_to_non_nullable
as LocalizedText?,department: freezed == department ? _self._department : department // ignore: cast_nullable_to_non_nullable
as LocalizedText?,kvk: freezed == kvk ? _self.kvk : kvk // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
