// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'maintenance_window.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MaintenanceWindow {

 DateTime get startDateTime; DateTime get endDateTime;
/// Create a copy of MaintenanceWindow
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MaintenanceWindowCopyWith<MaintenanceWindow> get copyWith => _$MaintenanceWindowCopyWithImpl<MaintenanceWindow>(this as MaintenanceWindow, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MaintenanceWindow&&(identical(other.startDateTime, startDateTime) || other.startDateTime == startDateTime)&&(identical(other.endDateTime, endDateTime) || other.endDateTime == endDateTime));
}


@override
int get hashCode => Object.hash(runtimeType,startDateTime,endDateTime);

@override
String toString() {
  return 'MaintenanceWindow(startDateTime: $startDateTime, endDateTime: $endDateTime)';
}


}

/// @nodoc
abstract mixin class $MaintenanceWindowCopyWith<$Res>  {
  factory $MaintenanceWindowCopyWith(MaintenanceWindow value, $Res Function(MaintenanceWindow) _then) = _$MaintenanceWindowCopyWithImpl;
@useResult
$Res call({
 DateTime startDateTime, DateTime endDateTime
});




}
/// @nodoc
class _$MaintenanceWindowCopyWithImpl<$Res>
    implements $MaintenanceWindowCopyWith<$Res> {
  _$MaintenanceWindowCopyWithImpl(this._self, this._then);

  final MaintenanceWindow _self;
  final $Res Function(MaintenanceWindow) _then;

/// Create a copy of MaintenanceWindow
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? startDateTime = null,Object? endDateTime = null,}) {
  return _then(_self.copyWith(
startDateTime: null == startDateTime ? _self.startDateTime : startDateTime // ignore: cast_nullable_to_non_nullable
as DateTime,endDateTime: null == endDateTime ? _self.endDateTime : endDateTime // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}

}


/// Adds pattern-matching-related methods to [MaintenanceWindow].
extension MaintenanceWindowPatterns on MaintenanceWindow {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MaintenanceWindow value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MaintenanceWindow() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MaintenanceWindow value)  $default,){
final _that = this;
switch (_that) {
case _MaintenanceWindow():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MaintenanceWindow value)?  $default,){
final _that = this;
switch (_that) {
case _MaintenanceWindow() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( DateTime startDateTime,  DateTime endDateTime)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MaintenanceWindow() when $default != null:
return $default(_that.startDateTime,_that.endDateTime);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( DateTime startDateTime,  DateTime endDateTime)  $default,) {final _that = this;
switch (_that) {
case _MaintenanceWindow():
return $default(_that.startDateTime,_that.endDateTime);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( DateTime startDateTime,  DateTime endDateTime)?  $default,) {final _that = this;
switch (_that) {
case _MaintenanceWindow() when $default != null:
return $default(_that.startDateTime,_that.endDateTime);case _:
  return null;

}
}

}

/// @nodoc


class _MaintenanceWindow extends MaintenanceWindow {
  const _MaintenanceWindow({required this.startDateTime, required this.endDateTime}): super._();
  

@override final  DateTime startDateTime;
@override final  DateTime endDateTime;

/// Create a copy of MaintenanceWindow
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MaintenanceWindowCopyWith<_MaintenanceWindow> get copyWith => __$MaintenanceWindowCopyWithImpl<_MaintenanceWindow>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MaintenanceWindow&&(identical(other.startDateTime, startDateTime) || other.startDateTime == startDateTime)&&(identical(other.endDateTime, endDateTime) || other.endDateTime == endDateTime));
}


@override
int get hashCode => Object.hash(runtimeType,startDateTime,endDateTime);

@override
String toString() {
  return 'MaintenanceWindow(startDateTime: $startDateTime, endDateTime: $endDateTime)';
}


}

/// @nodoc
abstract mixin class _$MaintenanceWindowCopyWith<$Res> implements $MaintenanceWindowCopyWith<$Res> {
  factory _$MaintenanceWindowCopyWith(_MaintenanceWindow value, $Res Function(_MaintenanceWindow) _then) = __$MaintenanceWindowCopyWithImpl;
@override @useResult
$Res call({
 DateTime startDateTime, DateTime endDateTime
});




}
/// @nodoc
class __$MaintenanceWindowCopyWithImpl<$Res>
    implements _$MaintenanceWindowCopyWith<$Res> {
  __$MaintenanceWindowCopyWithImpl(this._self, this._then);

  final _MaintenanceWindow _self;
  final $Res Function(_MaintenanceWindow) _then;

/// Create a copy of MaintenanceWindow
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? startDateTime = null,Object? endDateTime = null,}) {
  return _then(_MaintenanceWindow(
startDateTime: null == startDateTime ? _self.startDateTime : startDateTime // ignore: cast_nullable_to_non_nullable
as DateTime,endDateTime: null == endDateTime ? _self.endDateTime : endDateTime // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}


}

// dart format on
