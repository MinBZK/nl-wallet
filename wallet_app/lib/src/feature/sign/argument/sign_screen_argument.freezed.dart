// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'sign_screen_argument.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$SignScreenArgument {

 String? get mockSessionId; String? get uri;
/// Create a copy of SignScreenArgument
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignScreenArgumentCopyWith<SignScreenArgument> get copyWith => _$SignScreenArgumentCopyWithImpl<SignScreenArgument>(this as SignScreenArgument, _$identity);

  /// Serializes this SignScreenArgument to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignScreenArgument&&(identical(other.mockSessionId, mockSessionId) || other.mockSessionId == mockSessionId)&&(identical(other.uri, uri) || other.uri == uri));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,mockSessionId,uri);

@override
String toString() {
  return 'SignScreenArgument(mockSessionId: $mockSessionId, uri: $uri)';
}


}

/// @nodoc
abstract mixin class $SignScreenArgumentCopyWith<$Res>  {
  factory $SignScreenArgumentCopyWith(SignScreenArgument value, $Res Function(SignScreenArgument) _then) = _$SignScreenArgumentCopyWithImpl;
@useResult
$Res call({
 String? mockSessionId, String? uri
});




}
/// @nodoc
class _$SignScreenArgumentCopyWithImpl<$Res>
    implements $SignScreenArgumentCopyWith<$Res> {
  _$SignScreenArgumentCopyWithImpl(this._self, this._then);

  final SignScreenArgument _self;
  final $Res Function(SignScreenArgument) _then;

/// Create a copy of SignScreenArgument
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? mockSessionId = freezed,Object? uri = freezed,}) {
  return _then(_self.copyWith(
mockSessionId: freezed == mockSessionId ? _self.mockSessionId : mockSessionId // ignore: cast_nullable_to_non_nullable
as String?,uri: freezed == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [SignScreenArgument].
extension SignScreenArgumentPatterns on SignScreenArgument {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SignScreenArgument value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SignScreenArgument() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SignScreenArgument value)  $default,){
final _that = this;
switch (_that) {
case _SignScreenArgument():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SignScreenArgument value)?  $default,){
final _that = this;
switch (_that) {
case _SignScreenArgument() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String? mockSessionId,  String? uri)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SignScreenArgument() when $default != null:
return $default(_that.mockSessionId,_that.uri);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String? mockSessionId,  String? uri)  $default,) {final _that = this;
switch (_that) {
case _SignScreenArgument():
return $default(_that.mockSessionId,_that.uri);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String? mockSessionId,  String? uri)?  $default,) {final _that = this;
switch (_that) {
case _SignScreenArgument() when $default != null:
return $default(_that.mockSessionId,_that.uri);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _SignScreenArgument implements SignScreenArgument {
  const _SignScreenArgument({this.mockSessionId, this.uri}): assert(mockSessionId != null || uri != null, 'Either a mockSessionId or a uri is needed to start signing');
  factory _SignScreenArgument.fromJson(Map<String, dynamic> json) => _$SignScreenArgumentFromJson(json);

@override final  String? mockSessionId;
@override final  String? uri;

/// Create a copy of SignScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SignScreenArgumentCopyWith<_SignScreenArgument> get copyWith => __$SignScreenArgumentCopyWithImpl<_SignScreenArgument>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$SignScreenArgumentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SignScreenArgument&&(identical(other.mockSessionId, mockSessionId) || other.mockSessionId == mockSessionId)&&(identical(other.uri, uri) || other.uri == uri));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,mockSessionId,uri);

@override
String toString() {
  return 'SignScreenArgument(mockSessionId: $mockSessionId, uri: $uri)';
}


}

/// @nodoc
abstract mixin class _$SignScreenArgumentCopyWith<$Res> implements $SignScreenArgumentCopyWith<$Res> {
  factory _$SignScreenArgumentCopyWith(_SignScreenArgument value, $Res Function(_SignScreenArgument) _then) = __$SignScreenArgumentCopyWithImpl;
@override @useResult
$Res call({
 String? mockSessionId, String? uri
});




}
/// @nodoc
class __$SignScreenArgumentCopyWithImpl<$Res>
    implements _$SignScreenArgumentCopyWith<$Res> {
  __$SignScreenArgumentCopyWithImpl(this._self, this._then);

  final _SignScreenArgument _self;
  final $Res Function(_SignScreenArgument) _then;

/// Create a copy of SignScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? mockSessionId = freezed,Object? uri = freezed,}) {
  return _then(_SignScreenArgument(
mockSessionId: freezed == mockSessionId ? _self.mockSessionId : mockSessionId // ignore: cast_nullable_to_non_nullable
as String?,uri: freezed == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
