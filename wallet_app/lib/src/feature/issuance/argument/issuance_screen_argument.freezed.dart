// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'issuance_screen_argument.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$IssuanceScreenArgument {

 String? get mockSessionId; bool get isQrCode; bool get isRefreshFlow; String? get uri;
/// Create a copy of IssuanceScreenArgument
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$IssuanceScreenArgumentCopyWith<IssuanceScreenArgument> get copyWith => _$IssuanceScreenArgumentCopyWithImpl<IssuanceScreenArgument>(this as IssuanceScreenArgument, _$identity);

  /// Serializes this IssuanceScreenArgument to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IssuanceScreenArgument&&(identical(other.mockSessionId, mockSessionId) || other.mockSessionId == mockSessionId)&&(identical(other.isQrCode, isQrCode) || other.isQrCode == isQrCode)&&(identical(other.isRefreshFlow, isRefreshFlow) || other.isRefreshFlow == isRefreshFlow)&&(identical(other.uri, uri) || other.uri == uri));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,mockSessionId,isQrCode,isRefreshFlow,uri);

@override
String toString() {
  return 'IssuanceScreenArgument(mockSessionId: $mockSessionId, isQrCode: $isQrCode, isRefreshFlow: $isRefreshFlow, uri: $uri)';
}


}

/// @nodoc
abstract mixin class $IssuanceScreenArgumentCopyWith<$Res>  {
  factory $IssuanceScreenArgumentCopyWith(IssuanceScreenArgument value, $Res Function(IssuanceScreenArgument) _then) = _$IssuanceScreenArgumentCopyWithImpl;
@useResult
$Res call({
 String? mockSessionId, bool isQrCode, bool isRefreshFlow, String? uri
});




}
/// @nodoc
class _$IssuanceScreenArgumentCopyWithImpl<$Res>
    implements $IssuanceScreenArgumentCopyWith<$Res> {
  _$IssuanceScreenArgumentCopyWithImpl(this._self, this._then);

  final IssuanceScreenArgument _self;
  final $Res Function(IssuanceScreenArgument) _then;

/// Create a copy of IssuanceScreenArgument
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? mockSessionId = freezed,Object? isQrCode = null,Object? isRefreshFlow = null,Object? uri = freezed,}) {
  return _then(_self.copyWith(
mockSessionId: freezed == mockSessionId ? _self.mockSessionId : mockSessionId // ignore: cast_nullable_to_non_nullable
as String?,isQrCode: null == isQrCode ? _self.isQrCode : isQrCode // ignore: cast_nullable_to_non_nullable
as bool,isRefreshFlow: null == isRefreshFlow ? _self.isRefreshFlow : isRefreshFlow // ignore: cast_nullable_to_non_nullable
as bool,uri: freezed == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [IssuanceScreenArgument].
extension IssuanceScreenArgumentPatterns on IssuanceScreenArgument {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _IssuanceScreenArgument value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _IssuanceScreenArgument() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _IssuanceScreenArgument value)  $default,){
final _that = this;
switch (_that) {
case _IssuanceScreenArgument():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _IssuanceScreenArgument value)?  $default,){
final _that = this;
switch (_that) {
case _IssuanceScreenArgument() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String? mockSessionId,  bool isQrCode,  bool isRefreshFlow,  String? uri)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _IssuanceScreenArgument() when $default != null:
return $default(_that.mockSessionId,_that.isQrCode,_that.isRefreshFlow,_that.uri);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String? mockSessionId,  bool isQrCode,  bool isRefreshFlow,  String? uri)  $default,) {final _that = this;
switch (_that) {
case _IssuanceScreenArgument():
return $default(_that.mockSessionId,_that.isQrCode,_that.isRefreshFlow,_that.uri);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String? mockSessionId,  bool isQrCode,  bool isRefreshFlow,  String? uri)?  $default,) {final _that = this;
switch (_that) {
case _IssuanceScreenArgument() when $default != null:
return $default(_that.mockSessionId,_that.isQrCode,_that.isRefreshFlow,_that.uri);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _IssuanceScreenArgument implements IssuanceScreenArgument {
  const _IssuanceScreenArgument({this.mockSessionId, required this.isQrCode, this.isRefreshFlow = false, this.uri}): assert(mockSessionId != null || uri != null, 'Either a mockSessionId or a uri is needed to start issuance');
  factory _IssuanceScreenArgument.fromJson(Map<String, dynamic> json) => _$IssuanceScreenArgumentFromJson(json);

@override final  String? mockSessionId;
@override final  bool isQrCode;
@override@JsonKey() final  bool isRefreshFlow;
@override final  String? uri;

/// Create a copy of IssuanceScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$IssuanceScreenArgumentCopyWith<_IssuanceScreenArgument> get copyWith => __$IssuanceScreenArgumentCopyWithImpl<_IssuanceScreenArgument>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$IssuanceScreenArgumentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _IssuanceScreenArgument&&(identical(other.mockSessionId, mockSessionId) || other.mockSessionId == mockSessionId)&&(identical(other.isQrCode, isQrCode) || other.isQrCode == isQrCode)&&(identical(other.isRefreshFlow, isRefreshFlow) || other.isRefreshFlow == isRefreshFlow)&&(identical(other.uri, uri) || other.uri == uri));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,mockSessionId,isQrCode,isRefreshFlow,uri);

@override
String toString() {
  return 'IssuanceScreenArgument(mockSessionId: $mockSessionId, isQrCode: $isQrCode, isRefreshFlow: $isRefreshFlow, uri: $uri)';
}


}

/// @nodoc
abstract mixin class _$IssuanceScreenArgumentCopyWith<$Res> implements $IssuanceScreenArgumentCopyWith<$Res> {
  factory _$IssuanceScreenArgumentCopyWith(_IssuanceScreenArgument value, $Res Function(_IssuanceScreenArgument) _then) = __$IssuanceScreenArgumentCopyWithImpl;
@override @useResult
$Res call({
 String? mockSessionId, bool isQrCode, bool isRefreshFlow, String? uri
});




}
/// @nodoc
class __$IssuanceScreenArgumentCopyWithImpl<$Res>
    implements _$IssuanceScreenArgumentCopyWith<$Res> {
  __$IssuanceScreenArgumentCopyWithImpl(this._self, this._then);

  final _IssuanceScreenArgument _self;
  final $Res Function(_IssuanceScreenArgument) _then;

/// Create a copy of IssuanceScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? mockSessionId = freezed,Object? isQrCode = null,Object? isRefreshFlow = null,Object? uri = freezed,}) {
  return _then(_IssuanceScreenArgument(
mockSessionId: freezed == mockSessionId ? _self.mockSessionId : mockSessionId // ignore: cast_nullable_to_non_nullable
as String?,isQrCode: null == isQrCode ? _self.isQrCode : isQrCode // ignore: cast_nullable_to_non_nullable
as bool,isRefreshFlow: null == isRefreshFlow ? _self.isRefreshFlow : isRefreshFlow // ignore: cast_nullable_to_non_nullable
as bool,uri: freezed == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
