// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'disclosure_screen_argument.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$DisclosureScreenArgument {

 String get uri; bool get isQrCode;
/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DisclosureScreenArgumentCopyWith<DisclosureScreenArgument> get copyWith => _$DisclosureScreenArgumentCopyWithImpl<DisclosureScreenArgument>(this as DisclosureScreenArgument, _$identity);

  /// Serializes this DisclosureScreenArgument to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DisclosureScreenArgument&&(identical(other.uri, uri) || other.uri == uri)&&(identical(other.isQrCode, isQrCode) || other.isQrCode == isQrCode));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,uri,isQrCode);

@override
String toString() {
  return 'DisclosureScreenArgument(uri: $uri, isQrCode: $isQrCode)';
}


}

/// @nodoc
abstract mixin class $DisclosureScreenArgumentCopyWith<$Res>  {
  factory $DisclosureScreenArgumentCopyWith(DisclosureScreenArgument value, $Res Function(DisclosureScreenArgument) _then) = _$DisclosureScreenArgumentCopyWithImpl;
@useResult
$Res call({
 String uri, bool isQrCode
});




}
/// @nodoc
class _$DisclosureScreenArgumentCopyWithImpl<$Res>
    implements $DisclosureScreenArgumentCopyWith<$Res> {
  _$DisclosureScreenArgumentCopyWithImpl(this._self, this._then);

  final DisclosureScreenArgument _self;
  final $Res Function(DisclosureScreenArgument) _then;

/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? uri = null,Object? isQrCode = null,}) {
  return _then(_self.copyWith(
uri: null == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String,isQrCode: null == isQrCode ? _self.isQrCode : isQrCode // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [DisclosureScreenArgument].
extension DisclosureScreenArgumentPatterns on DisclosureScreenArgument {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DisclosureScreenArgument value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DisclosureScreenArgument() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DisclosureScreenArgument value)  $default,){
final _that = this;
switch (_that) {
case _DisclosureScreenArgument():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DisclosureScreenArgument value)?  $default,){
final _that = this;
switch (_that) {
case _DisclosureScreenArgument() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String uri,  bool isQrCode)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DisclosureScreenArgument() when $default != null:
return $default(_that.uri,_that.isQrCode);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String uri,  bool isQrCode)  $default,) {final _that = this;
switch (_that) {
case _DisclosureScreenArgument():
return $default(_that.uri,_that.isQrCode);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String uri,  bool isQrCode)?  $default,) {final _that = this;
switch (_that) {
case _DisclosureScreenArgument() when $default != null:
return $default(_that.uri,_that.isQrCode);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DisclosureScreenArgument implements DisclosureScreenArgument {
  const _DisclosureScreenArgument({required this.uri, required this.isQrCode});
  factory _DisclosureScreenArgument.fromJson(Map<String, dynamic> json) => _$DisclosureScreenArgumentFromJson(json);

@override final  String uri;
@override final  bool isQrCode;

/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DisclosureScreenArgumentCopyWith<_DisclosureScreenArgument> get copyWith => __$DisclosureScreenArgumentCopyWithImpl<_DisclosureScreenArgument>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DisclosureScreenArgumentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DisclosureScreenArgument&&(identical(other.uri, uri) || other.uri == uri)&&(identical(other.isQrCode, isQrCode) || other.isQrCode == isQrCode));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,uri,isQrCode);

@override
String toString() {
  return 'DisclosureScreenArgument(uri: $uri, isQrCode: $isQrCode)';
}


}

/// @nodoc
abstract mixin class _$DisclosureScreenArgumentCopyWith<$Res> implements $DisclosureScreenArgumentCopyWith<$Res> {
  factory _$DisclosureScreenArgumentCopyWith(_DisclosureScreenArgument value, $Res Function(_DisclosureScreenArgument) _then) = __$DisclosureScreenArgumentCopyWithImpl;
@override @useResult
$Res call({
 String uri, bool isQrCode
});




}
/// @nodoc
class __$DisclosureScreenArgumentCopyWithImpl<$Res>
    implements _$DisclosureScreenArgumentCopyWith<$Res> {
  __$DisclosureScreenArgumentCopyWithImpl(this._self, this._then);

  final _DisclosureScreenArgument _self;
  final $Res Function(_DisclosureScreenArgument) _then;

/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? uri = null,Object? isQrCode = null,}) {
  return _then(_DisclosureScreenArgument(
uri: null == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String,isQrCode: null == isQrCode ? _self.isQrCode : isQrCode // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
