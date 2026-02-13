// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'maintenance_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MaintenanceState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MaintenanceState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MaintenanceState()';
}


}

/// @nodoc
class $MaintenanceStateCopyWith<$Res>  {
$MaintenanceStateCopyWith(MaintenanceState _, $Res Function(MaintenanceState) __);
}


/// Adds pattern-matching-related methods to [MaintenanceState].
extension MaintenanceStatePatterns on MaintenanceState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( InMaintenance value)?  inMaintenance,TResult Function( NoMaintenance value)?  noMaintenance,required TResult orElse(),}){
final _that = this;
switch (_that) {
case InMaintenance() when inMaintenance != null:
return inMaintenance(_that);case NoMaintenance() when noMaintenance != null:
return noMaintenance(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( InMaintenance value)  inMaintenance,required TResult Function( NoMaintenance value)  noMaintenance,}){
final _that = this;
switch (_that) {
case InMaintenance():
return inMaintenance(_that);case NoMaintenance():
return noMaintenance(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( InMaintenance value)?  inMaintenance,TResult? Function( NoMaintenance value)?  noMaintenance,}){
final _that = this;
switch (_that) {
case InMaintenance() when inMaintenance != null:
return inMaintenance(_that);case NoMaintenance() when noMaintenance != null:
return noMaintenance(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( MaintenanceWindow window)?  inMaintenance,TResult Function()?  noMaintenance,required TResult orElse(),}) {final _that = this;
switch (_that) {
case InMaintenance() when inMaintenance != null:
return inMaintenance(_that.window);case NoMaintenance() when noMaintenance != null:
return noMaintenance();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( MaintenanceWindow window)  inMaintenance,required TResult Function()  noMaintenance,}) {final _that = this;
switch (_that) {
case InMaintenance():
return inMaintenance(_that.window);case NoMaintenance():
return noMaintenance();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( MaintenanceWindow window)?  inMaintenance,TResult? Function()?  noMaintenance,}) {final _that = this;
switch (_that) {
case InMaintenance() when inMaintenance != null:
return inMaintenance(_that.window);case NoMaintenance() when noMaintenance != null:
return noMaintenance();case _:
  return null;

}
}

}

/// @nodoc


class InMaintenance implements MaintenanceState {
  const InMaintenance(this.window);
  

 final  MaintenanceWindow window;

/// Create a copy of MaintenanceState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InMaintenanceCopyWith<InMaintenance> get copyWith => _$InMaintenanceCopyWithImpl<InMaintenance>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InMaintenance&&(identical(other.window, window) || other.window == window));
}


@override
int get hashCode => Object.hash(runtimeType,window);

@override
String toString() {
  return 'MaintenanceState.inMaintenance(window: $window)';
}


}

/// @nodoc
abstract mixin class $InMaintenanceCopyWith<$Res> implements $MaintenanceStateCopyWith<$Res> {
  factory $InMaintenanceCopyWith(InMaintenance value, $Res Function(InMaintenance) _then) = _$InMaintenanceCopyWithImpl;
@useResult
$Res call({
 MaintenanceWindow window
});


$MaintenanceWindowCopyWith<$Res> get window;

}
/// @nodoc
class _$InMaintenanceCopyWithImpl<$Res>
    implements $InMaintenanceCopyWith<$Res> {
  _$InMaintenanceCopyWithImpl(this._self, this._then);

  final InMaintenance _self;
  final $Res Function(InMaintenance) _then;

/// Create a copy of MaintenanceState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? window = null,}) {
  return _then(InMaintenance(
null == window ? _self.window : window // ignore: cast_nullable_to_non_nullable
as MaintenanceWindow,
  ));
}

/// Create a copy of MaintenanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$MaintenanceWindowCopyWith<$Res> get window {
  
  return $MaintenanceWindowCopyWith<$Res>(_self.window, (value) {
    return _then(_self.copyWith(window: value));
  });
}
}

/// @nodoc


class NoMaintenance implements MaintenanceState {
  const NoMaintenance();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoMaintenance);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MaintenanceState.noMaintenance()';
}


}




// dart format on
