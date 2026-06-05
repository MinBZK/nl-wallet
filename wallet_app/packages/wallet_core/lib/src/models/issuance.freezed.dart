// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'issuance.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$IssuanceStartResult {

 Object get field0;



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IssuanceStartResult&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'IssuanceStartResult(field0: $field0)';
}


}

/// @nodoc
class $IssuanceStartResultCopyWith<$Res>  {
$IssuanceStartResultCopyWith(IssuanceStartResult _, $Res Function(IssuanceStartResult) __);
}


/// Adds pattern-matching-related methods to [IssuanceStartResult].
extension IssuanceStartResultPatterns on IssuanceStartResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( IssuanceStartResult_AuthorizationUrl value)?  authorizationUrl,TResult Function( IssuanceStartResult_Previews value)?  previews,required TResult orElse(),}){
final _that = this;
switch (_that) {
case IssuanceStartResult_AuthorizationUrl() when authorizationUrl != null:
return authorizationUrl(_that);case IssuanceStartResult_Previews() when previews != null:
return previews(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( IssuanceStartResult_AuthorizationUrl value)  authorizationUrl,required TResult Function( IssuanceStartResult_Previews value)  previews,}){
final _that = this;
switch (_that) {
case IssuanceStartResult_AuthorizationUrl():
return authorizationUrl(_that);case IssuanceStartResult_Previews():
return previews(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( IssuanceStartResult_AuthorizationUrl value)?  authorizationUrl,TResult? Function( IssuanceStartResult_Previews value)?  previews,}){
final _that = this;
switch (_that) {
case IssuanceStartResult_AuthorizationUrl() when authorizationUrl != null:
return authorizationUrl(_that);case IssuanceStartResult_Previews() when previews != null:
return previews(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String field0)?  authorizationUrl,TResult Function( List<AttestationPresentation> field0)?  previews,required TResult orElse(),}) {final _that = this;
switch (_that) {
case IssuanceStartResult_AuthorizationUrl() when authorizationUrl != null:
return authorizationUrl(_that.field0);case IssuanceStartResult_Previews() when previews != null:
return previews(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String field0)  authorizationUrl,required TResult Function( List<AttestationPresentation> field0)  previews,}) {final _that = this;
switch (_that) {
case IssuanceStartResult_AuthorizationUrl():
return authorizationUrl(_that.field0);case IssuanceStartResult_Previews():
return previews(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String field0)?  authorizationUrl,TResult? Function( List<AttestationPresentation> field0)?  previews,}) {final _that = this;
switch (_that) {
case IssuanceStartResult_AuthorizationUrl() when authorizationUrl != null:
return authorizationUrl(_that.field0);case IssuanceStartResult_Previews() when previews != null:
return previews(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class IssuanceStartResult_AuthorizationUrl extends IssuanceStartResult {
  const IssuanceStartResult_AuthorizationUrl(this.field0): super._();
  

@override final  String field0;

/// Create a copy of IssuanceStartResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$IssuanceStartResult_AuthorizationUrlCopyWith<IssuanceStartResult_AuthorizationUrl> get copyWith => _$IssuanceStartResult_AuthorizationUrlCopyWithImpl<IssuanceStartResult_AuthorizationUrl>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IssuanceStartResult_AuthorizationUrl&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'IssuanceStartResult.authorizationUrl(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $IssuanceStartResult_AuthorizationUrlCopyWith<$Res> implements $IssuanceStartResultCopyWith<$Res> {
  factory $IssuanceStartResult_AuthorizationUrlCopyWith(IssuanceStartResult_AuthorizationUrl value, $Res Function(IssuanceStartResult_AuthorizationUrl) _then) = _$IssuanceStartResult_AuthorizationUrlCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$IssuanceStartResult_AuthorizationUrlCopyWithImpl<$Res>
    implements $IssuanceStartResult_AuthorizationUrlCopyWith<$Res> {
  _$IssuanceStartResult_AuthorizationUrlCopyWithImpl(this._self, this._then);

  final IssuanceStartResult_AuthorizationUrl _self;
  final $Res Function(IssuanceStartResult_AuthorizationUrl) _then;

/// Create a copy of IssuanceStartResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(IssuanceStartResult_AuthorizationUrl(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class IssuanceStartResult_Previews extends IssuanceStartResult {
  const IssuanceStartResult_Previews(final  List<AttestationPresentation> field0): _field0 = field0,super._();
  

 final  List<AttestationPresentation> _field0;
@override List<AttestationPresentation> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of IssuanceStartResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$IssuanceStartResult_PreviewsCopyWith<IssuanceStartResult_Previews> get copyWith => _$IssuanceStartResult_PreviewsCopyWithImpl<IssuanceStartResult_Previews>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IssuanceStartResult_Previews&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'IssuanceStartResult.previews(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $IssuanceStartResult_PreviewsCopyWith<$Res> implements $IssuanceStartResultCopyWith<$Res> {
  factory $IssuanceStartResult_PreviewsCopyWith(IssuanceStartResult_Previews value, $Res Function(IssuanceStartResult_Previews) _then) = _$IssuanceStartResult_PreviewsCopyWithImpl;
@useResult
$Res call({
 List<AttestationPresentation> field0
});




}
/// @nodoc
class _$IssuanceStartResult_PreviewsCopyWithImpl<$Res>
    implements $IssuanceStartResult_PreviewsCopyWith<$Res> {
  _$IssuanceStartResult_PreviewsCopyWithImpl(this._self, this._then);

  final IssuanceStartResult_Previews _self;
  final $Res Function(IssuanceStartResult_Previews) _then;

/// Create a copy of IssuanceStartResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(IssuanceStartResult_Previews(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<AttestationPresentation>,
  ));
}


}

// dart format on
