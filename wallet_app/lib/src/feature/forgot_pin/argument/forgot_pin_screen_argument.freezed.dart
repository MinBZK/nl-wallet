// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'forgot_pin_screen_argument.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$ForgotPinScreenArgument {

 bool get useCloseButton;
/// Create a copy of ForgotPinScreenArgument
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ForgotPinScreenArgumentCopyWith<ForgotPinScreenArgument> get copyWith => _$ForgotPinScreenArgumentCopyWithImpl<ForgotPinScreenArgument>(this as ForgotPinScreenArgument, _$identity);

  /// Serializes this ForgotPinScreenArgument to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ForgotPinScreenArgument&&(identical(other.useCloseButton, useCloseButton) || other.useCloseButton == useCloseButton));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,useCloseButton);

@override
String toString() {
  return 'ForgotPinScreenArgument(useCloseButton: $useCloseButton)';
}


}

/// @nodoc
abstract mixin class $ForgotPinScreenArgumentCopyWith<$Res>  {
  factory $ForgotPinScreenArgumentCopyWith(ForgotPinScreenArgument value, $Res Function(ForgotPinScreenArgument) _then) = _$ForgotPinScreenArgumentCopyWithImpl;
@useResult
$Res call({
 bool useCloseButton
});




}
/// @nodoc
class _$ForgotPinScreenArgumentCopyWithImpl<$Res>
    implements $ForgotPinScreenArgumentCopyWith<$Res> {
  _$ForgotPinScreenArgumentCopyWithImpl(this._self, this._then);

  final ForgotPinScreenArgument _self;
  final $Res Function(ForgotPinScreenArgument) _then;

/// Create a copy of ForgotPinScreenArgument
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? useCloseButton = null,}) {
  return _then(_self.copyWith(
useCloseButton: null == useCloseButton ? _self.useCloseButton : useCloseButton // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [ForgotPinScreenArgument].
extension ForgotPinScreenArgumentPatterns on ForgotPinScreenArgument {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ForgotPinScreenArgument value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ForgotPinScreenArgument() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ForgotPinScreenArgument value)  $default,){
final _that = this;
switch (_that) {
case _ForgotPinScreenArgument():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ForgotPinScreenArgument value)?  $default,){
final _that = this;
switch (_that) {
case _ForgotPinScreenArgument() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool useCloseButton)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ForgotPinScreenArgument() when $default != null:
return $default(_that.useCloseButton);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool useCloseButton)  $default,) {final _that = this;
switch (_that) {
case _ForgotPinScreenArgument():
return $default(_that.useCloseButton);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool useCloseButton)?  $default,) {final _that = this;
switch (_that) {
case _ForgotPinScreenArgument() when $default != null:
return $default(_that.useCloseButton);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ForgotPinScreenArgument implements ForgotPinScreenArgument {
  const _ForgotPinScreenArgument({required this.useCloseButton});
  factory _ForgotPinScreenArgument.fromJson(Map<String, dynamic> json) => _$ForgotPinScreenArgumentFromJson(json);

@override final  bool useCloseButton;

/// Create a copy of ForgotPinScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ForgotPinScreenArgumentCopyWith<_ForgotPinScreenArgument> get copyWith => __$ForgotPinScreenArgumentCopyWithImpl<_ForgotPinScreenArgument>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ForgotPinScreenArgumentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ForgotPinScreenArgument&&(identical(other.useCloseButton, useCloseButton) || other.useCloseButton == useCloseButton));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,useCloseButton);

@override
String toString() {
  return 'ForgotPinScreenArgument(useCloseButton: $useCloseButton)';
}


}

/// @nodoc
abstract mixin class _$ForgotPinScreenArgumentCopyWith<$Res> implements $ForgotPinScreenArgumentCopyWith<$Res> {
  factory _$ForgotPinScreenArgumentCopyWith(_ForgotPinScreenArgument value, $Res Function(_ForgotPinScreenArgument) _then) = __$ForgotPinScreenArgumentCopyWithImpl;
@override @useResult
$Res call({
 bool useCloseButton
});




}
/// @nodoc
class __$ForgotPinScreenArgumentCopyWithImpl<$Res>
    implements _$ForgotPinScreenArgumentCopyWith<$Res> {
  __$ForgotPinScreenArgumentCopyWithImpl(this._self, this._then);

  final _ForgotPinScreenArgument _self;
  final $Res Function(_ForgotPinScreenArgument) _then;

/// Create a copy of ForgotPinScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? useCloseButton = null,}) {
  return _then(_ForgotPinScreenArgument(
useCloseButton: null == useCloseButton ? _self.useCloseButton : useCloseButton // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
