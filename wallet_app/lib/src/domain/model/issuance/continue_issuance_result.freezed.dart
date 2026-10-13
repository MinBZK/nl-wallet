// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'continue_issuance_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ContinueIssuanceResult {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ContinueIssuanceResult);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'ContinueIssuanceResult()';
}


}

/// @nodoc
class $ContinueIssuanceResultCopyWith<$Res>  {
$ContinueIssuanceResultCopyWith(ContinueIssuanceResult _, $Res Function(ContinueIssuanceResult) __);
}


/// Adds pattern-matching-related methods to [ContinueIssuanceResult].
extension ContinueIssuanceResultPatterns on ContinueIssuanceResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ContinueIssuancePreviews value)?  previews,TResult Function( ContinueIssuanceNoPreviews value)?  noPreviews,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ContinueIssuancePreviews() when previews != null:
return previews(_that);case ContinueIssuanceNoPreviews() when noPreviews != null:
return noPreviews(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ContinueIssuancePreviews value)  previews,required TResult Function( ContinueIssuanceNoPreviews value)  noPreviews,}){
final _that = this;
switch (_that) {
case ContinueIssuancePreviews():
return previews(_that);case ContinueIssuanceNoPreviews():
return noPreviews(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ContinueIssuancePreviews value)?  previews,TResult? Function( ContinueIssuanceNoPreviews value)?  noPreviews,}){
final _that = this;
switch (_that) {
case ContinueIssuancePreviews() when previews != null:
return previews(_that);case ContinueIssuanceNoPreviews() when noPreviews != null:
return noPreviews(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( List<WalletCard> previews)?  previews,TResult Function( Organization organization)?  noPreviews,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ContinueIssuancePreviews() when previews != null:
return previews(_that.previews);case ContinueIssuanceNoPreviews() when noPreviews != null:
return noPreviews(_that.organization);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( List<WalletCard> previews)  previews,required TResult Function( Organization organization)  noPreviews,}) {final _that = this;
switch (_that) {
case ContinueIssuancePreviews():
return previews(_that.previews);case ContinueIssuanceNoPreviews():
return noPreviews(_that.organization);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( List<WalletCard> previews)?  previews,TResult? Function( Organization organization)?  noPreviews,}) {final _that = this;
switch (_that) {
case ContinueIssuancePreviews() when previews != null:
return previews(_that.previews);case ContinueIssuanceNoPreviews() when noPreviews != null:
return noPreviews(_that.organization);case _:
  return null;

}
}

}

/// @nodoc


class ContinueIssuancePreviews implements ContinueIssuanceResult {
  const ContinueIssuancePreviews( List<WalletCard> previews): _previews = previews;
  

 final  List<WalletCard> _previews;
 List<WalletCard> get previews {
  if (_previews is EqualUnmodifiableListView) return _previews;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_previews);
}


/// Create a copy of ContinueIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ContinueIssuancePreviewsCopyWith<ContinueIssuancePreviews> get copyWith => _$ContinueIssuancePreviewsCopyWithImpl<ContinueIssuancePreviews>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ContinueIssuancePreviews&&const DeepCollectionEquality().equals(other.previews, _previews));
}


@override
int get hashCode {
    return Object.hash(runtimeType,const DeepCollectionEquality().hash(_previews));
}

@override
String toString() {
    return 'ContinueIssuanceResult.previews(previews: $previews)';
}


}

/// @nodoc
abstract mixin class $ContinueIssuancePreviewsCopyWith<$Res> implements $ContinueIssuanceResultCopyWith<$Res> {
  factory $ContinueIssuancePreviewsCopyWith(ContinueIssuancePreviews value, $Res Function(ContinueIssuancePreviews) _then) = _$ContinueIssuancePreviewsCopyWithImpl;
@useResult
$Res call({
 List<WalletCard> previews
});




}
/// @nodoc
class _$ContinueIssuancePreviewsCopyWithImpl<$Res>
    implements $ContinueIssuancePreviewsCopyWith<$Res> {
  _$ContinueIssuancePreviewsCopyWithImpl(this._self, this._then);

  final ContinueIssuancePreviews _self;
  final $Res Function(ContinueIssuancePreviews) _then;

/// Create a copy of ContinueIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? previews = null,}) {
  return _then(ContinueIssuancePreviews(
null == previews ? _self._previews : previews // ignore: cast_nullable_to_non_nullable
as List<WalletCard>,
  ));
}


}

/// @nodoc


class ContinueIssuanceNoPreviews implements ContinueIssuanceResult {
  const ContinueIssuanceNoPreviews({required this.organization});
  

 final  Organization organization;

/// Create a copy of ContinueIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ContinueIssuanceNoPreviewsCopyWith<ContinueIssuanceNoPreviews> get copyWith => _$ContinueIssuanceNoPreviewsCopyWithImpl<ContinueIssuanceNoPreviews>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ContinueIssuanceNoPreviews&&(identical(other.organization, organization) || other.organization == organization));
}


@override
int get hashCode {
    return Object.hash(runtimeType,organization);
}

@override
String toString() {
    return 'ContinueIssuanceResult.noPreviews(organization: $organization)';
}


}

/// @nodoc
abstract mixin class $ContinueIssuanceNoPreviewsCopyWith<$Res> implements $ContinueIssuanceResultCopyWith<$Res> {
  factory $ContinueIssuanceNoPreviewsCopyWith(ContinueIssuanceNoPreviews value, $Res Function(ContinueIssuanceNoPreviews) _then) = _$ContinueIssuanceNoPreviewsCopyWithImpl;
@useResult
$Res call({
 Organization organization
});


$OrganizationCopyWith<$Res> get organization;

}
/// @nodoc
class _$ContinueIssuanceNoPreviewsCopyWithImpl<$Res>
    implements $ContinueIssuanceNoPreviewsCopyWith<$Res> {
  _$ContinueIssuanceNoPreviewsCopyWithImpl(this._self, this._then);

  final ContinueIssuanceNoPreviews _self;
  final $Res Function(ContinueIssuanceNoPreviews) _then;

/// Create a copy of ContinueIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? organization = null,}) {
  return _then(ContinueIssuanceNoPreviews(
organization: null == organization ? _self.organization : organization // ignore: cast_nullable_to_non_nullable
as Organization,
  ));
}

/// Create a copy of ContinueIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OrganizationCopyWith<$Res> get organization {
  
  return $OrganizationCopyWith<$Res>(_self.organization, (value) {
    return _then(_self.copyWith(organization: value));
  });
}
}

// dart format on
