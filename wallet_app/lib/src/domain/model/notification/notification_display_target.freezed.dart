// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'notification_display_target.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NotificationDisplayTarget {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotificationDisplayTarget);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotificationDisplayTarget()';
}


}

/// @nodoc
class $NotificationDisplayTargetCopyWith<$Res>  {
$NotificationDisplayTargetCopyWith(NotificationDisplayTarget _, $Res Function(NotificationDisplayTarget) __);
}


/// Adds pattern-matching-related methods to [NotificationDisplayTarget].
extension NotificationDisplayTargetPatterns on NotificationDisplayTarget {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( Os value)?  os,TResult Function( Dashboard value)?  dashboard,required TResult orElse(),}){
final _that = this;
switch (_that) {
case Os() when os != null:
return os(_that);case Dashboard() when dashboard != null:
return dashboard(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( Os value)  os,required TResult Function( Dashboard value)  dashboard,}){
final _that = this;
switch (_that) {
case Os():
return os(_that);case Dashboard():
return dashboard(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( Os value)?  os,TResult? Function( Dashboard value)?  dashboard,}){
final _that = this;
switch (_that) {
case Os() when os != null:
return os(_that);case Dashboard() when dashboard != null:
return dashboard(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( DateTime notifyAt)?  os,TResult Function()?  dashboard,required TResult orElse(),}) {final _that = this;
switch (_that) {
case Os() when os != null:
return os(_that.notifyAt);case Dashboard() when dashboard != null:
return dashboard();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( DateTime notifyAt)  os,required TResult Function()  dashboard,}) {final _that = this;
switch (_that) {
case Os():
return os(_that.notifyAt);case Dashboard():
return dashboard();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( DateTime notifyAt)?  os,TResult? Function()?  dashboard,}) {final _that = this;
switch (_that) {
case Os() when os != null:
return os(_that.notifyAt);case Dashboard() when dashboard != null:
return dashboard();case _:
  return null;

}
}

}

/// @nodoc


class Os implements NotificationDisplayTarget {
  const Os({required this.notifyAt});
  

 final  DateTime notifyAt;

/// Create a copy of NotificationDisplayTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OsCopyWith<Os> get copyWith => _$OsCopyWithImpl<Os>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Os&&(identical(other.notifyAt, notifyAt) || other.notifyAt == notifyAt));
}


@override
int get hashCode => Object.hash(runtimeType,notifyAt);

@override
String toString() {
  return 'NotificationDisplayTarget.os(notifyAt: $notifyAt)';
}


}

/// @nodoc
abstract mixin class $OsCopyWith<$Res> implements $NotificationDisplayTargetCopyWith<$Res> {
  factory $OsCopyWith(Os value, $Res Function(Os) _then) = _$OsCopyWithImpl;
@useResult
$Res call({
 DateTime notifyAt
});




}
/// @nodoc
class _$OsCopyWithImpl<$Res>
    implements $OsCopyWith<$Res> {
  _$OsCopyWithImpl(this._self, this._then);

  final Os _self;
  final $Res Function(Os) _then;

/// Create a copy of NotificationDisplayTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notifyAt = null,}) {
  return _then(Os(
notifyAt: null == notifyAt ? _self.notifyAt : notifyAt // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}


}

/// @nodoc


class Dashboard implements NotificationDisplayTarget {
  const Dashboard();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Dashboard);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotificationDisplayTarget.dashboard()';
}


}




// dart format on
