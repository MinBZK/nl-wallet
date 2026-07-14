// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'recover_pin_screen_argument.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$RecoverPinScreenArgument {

 String? get uri; bool get isRecoveryFlow;

  /// Serializes this RecoverPinScreenArgument to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoverPinScreenArgument&&(identical(other.uri, uri) || other.uri == uri)&&(identical(other.isRecoveryFlow, isRecoveryFlow) || other.isRecoveryFlow == isRecoveryFlow));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,uri,isRecoveryFlow);

@override
String toString() {
  return 'RecoverPinScreenArgument(uri: $uri, isRecoveryFlow: $isRecoveryFlow)';
}


}




/// Adds pattern-matching-related methods to [RecoverPinScreenArgument].
extension RecoverPinScreenArgumentPatterns on RecoverPinScreenArgument {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _RecoverPinScreenArgument value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _RecoverPinScreenArgument() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _RecoverPinScreenArgument value)  $default,){
final _that = this;
switch (_that) {
case _RecoverPinScreenArgument():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _RecoverPinScreenArgument value)?  $default,){
final _that = this;
switch (_that) {
case _RecoverPinScreenArgument() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String? uri,  bool isRecoveryFlow)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _RecoverPinScreenArgument() when $default != null:
return $default(_that.uri,_that.isRecoveryFlow);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String? uri,  bool isRecoveryFlow)  $default,) {final _that = this;
switch (_that) {
case _RecoverPinScreenArgument():
return $default(_that.uri,_that.isRecoveryFlow);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String? uri,  bool isRecoveryFlow)?  $default,) {final _that = this;
switch (_that) {
case _RecoverPinScreenArgument() when $default != null:
return $default(_that.uri,_that.isRecoveryFlow);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _RecoverPinScreenArgument extends RecoverPinScreenArgument {
  const _RecoverPinScreenArgument({this.uri, this.isRecoveryFlow = false}): super._();
  factory _RecoverPinScreenArgument.fromJson(Map<String, dynamic> json) => _$RecoverPinScreenArgumentFromJson(json);

@override final  String? uri;
@override@JsonKey() final  bool isRecoveryFlow;


@override
Map<String, dynamic> toJson() {
  return _$RecoverPinScreenArgumentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _RecoverPinScreenArgument&&(identical(other.uri, uri) || other.uri == uri)&&(identical(other.isRecoveryFlow, isRecoveryFlow) || other.isRecoveryFlow == isRecoveryFlow));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,uri,isRecoveryFlow);

@override
String toString() {
  return 'RecoverPinScreenArgument(uri: $uri, isRecoveryFlow: $isRecoveryFlow)';
}


}




// dart format on
