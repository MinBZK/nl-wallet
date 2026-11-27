// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'continue_pid_issuance_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ContinuePidIssuanceResult {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ContinuePidIssuanceResult);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'ContinuePidIssuanceResult()';
}


}

/// @nodoc
class $ContinuePidIssuanceResultCopyWith<$Res>  {
$ContinuePidIssuanceResultCopyWith(ContinuePidIssuanceResult _, $Res Function(ContinuePidIssuanceResult) __);
}


/// Adds pattern-matching-related methods to [ContinuePidIssuanceResult].
extension ContinuePidIssuanceResultPatterns on ContinuePidIssuanceResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ContinuePidIssuancePreviews value)?  previews,TResult Function( ContinuePidIssuanceNoPreviews value)?  noPreviews,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ContinuePidIssuancePreviews() when previews != null:
return previews(_that);case ContinuePidIssuanceNoPreviews() when noPreviews != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ContinuePidIssuancePreviews value)  previews,required TResult Function( ContinuePidIssuanceNoPreviews value)  noPreviews,}){
final _that = this;
switch (_that) {
case ContinuePidIssuancePreviews():
return previews(_that);case ContinuePidIssuanceNoPreviews():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ContinuePidIssuancePreviews value)?  previews,TResult? Function( ContinuePidIssuanceNoPreviews value)?  noPreviews,}){
final _that = this;
switch (_that) {
case ContinuePidIssuancePreviews() when previews != null:
return previews(_that);case ContinuePidIssuanceNoPreviews() when noPreviews != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( List<Attribute> attributes)?  previews,TResult Function()?  noPreviews,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ContinuePidIssuancePreviews() when previews != null:
return previews(_that.attributes);case ContinuePidIssuanceNoPreviews() when noPreviews != null:
return noPreviews();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( List<Attribute> attributes)  previews,required TResult Function()  noPreviews,}) {final _that = this;
switch (_that) {
case ContinuePidIssuancePreviews():
return previews(_that.attributes);case ContinuePidIssuanceNoPreviews():
return noPreviews();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( List<Attribute> attributes)?  previews,TResult? Function()?  noPreviews,}) {final _that = this;
switch (_that) {
case ContinuePidIssuancePreviews() when previews != null:
return previews(_that.attributes);case ContinuePidIssuanceNoPreviews() when noPreviews != null:
return noPreviews();case _:
  return null;

}
}

}

/// @nodoc


class ContinuePidIssuancePreviews implements ContinuePidIssuanceResult {
  const ContinuePidIssuancePreviews( List<Attribute> attributes): _attributes = attributes;
  

 final  List<Attribute> _attributes;
 List<Attribute> get attributes {
  if (_attributes is EqualUnmodifiableListView) return _attributes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_attributes);
}


/// Create a copy of ContinuePidIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ContinuePidIssuancePreviewsCopyWith<ContinuePidIssuancePreviews> get copyWith => _$ContinuePidIssuancePreviewsCopyWithImpl<ContinuePidIssuancePreviews>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ContinuePidIssuancePreviews&&const DeepCollectionEquality().equals(other.attributes, _attributes));
}


@override
int get hashCode {
    return Object.hash(runtimeType,const DeepCollectionEquality().hash(_attributes));
}

@override
String toString() {
    return 'ContinuePidIssuanceResult.previews(attributes: $attributes)';
}


}

/// @nodoc
abstract mixin class $ContinuePidIssuancePreviewsCopyWith<$Res> implements $ContinuePidIssuanceResultCopyWith<$Res> {
  factory $ContinuePidIssuancePreviewsCopyWith(ContinuePidIssuancePreviews value, $Res Function(ContinuePidIssuancePreviews) _then) = _$ContinuePidIssuancePreviewsCopyWithImpl;
@useResult
$Res call({
 List<Attribute> attributes
});




}
/// @nodoc
class _$ContinuePidIssuancePreviewsCopyWithImpl<$Res>
    implements $ContinuePidIssuancePreviewsCopyWith<$Res> {
  _$ContinuePidIssuancePreviewsCopyWithImpl(this._self, this._then);

  final ContinuePidIssuancePreviews _self;
  final $Res Function(ContinuePidIssuancePreviews) _then;

/// Create a copy of ContinuePidIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? attributes = null,}) {
  return _then(ContinuePidIssuancePreviews(
null == attributes ? _self._attributes : attributes // ignore: cast_nullable_to_non_nullable
as List<Attribute>,
  ));
}


}

/// @nodoc


class ContinuePidIssuanceNoPreviews implements ContinuePidIssuanceResult {
  const ContinuePidIssuanceNoPreviews();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ContinuePidIssuanceNoPreviews);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'ContinuePidIssuanceResult.noPreviews()';
}


}




// dart format on
