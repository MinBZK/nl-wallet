// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'os_notification.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$OsNotification {

/// A unique identifier for this notification.
 int get id;/// The channel through which this notification will be delivered.
///
/// Channels are used on modern Android versions to group notifications and
/// allow users to manage them.
 NotificationChannel get channel;/// The localized title of the notification.
 String get title;/// The localized body of the notification.
 String get body;/// The payload of the notification.
 String? get payload;/// The exact date and time when the notification should be displayed.
 DateTime get notifyAt;
/// Create a copy of OsNotification
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OsNotificationCopyWith<OsNotification> get copyWith => _$OsNotificationCopyWithImpl<OsNotification>(this as OsNotification, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OsNotification&&(identical(other.id, id) || other.id == id)&&(identical(other.channel, channel) || other.channel == channel)&&(identical(other.title, title) || other.title == title)&&(identical(other.body, body) || other.body == body)&&(identical(other.payload, payload) || other.payload == payload)&&(identical(other.notifyAt, notifyAt) || other.notifyAt == notifyAt));
}


@override
int get hashCode => Object.hash(runtimeType,id,channel,title,body,payload,notifyAt);

@override
String toString() {
  return 'OsNotification(id: $id, channel: $channel, title: $title, body: $body, payload: $payload, notifyAt: $notifyAt)';
}


}

/// @nodoc
abstract mixin class $OsNotificationCopyWith<$Res>  {
  factory $OsNotificationCopyWith(OsNotification value, $Res Function(OsNotification) _then) = _$OsNotificationCopyWithImpl;
@useResult
$Res call({
 int id, NotificationChannel channel, String title, String body, String? payload, DateTime notifyAt
});




}
/// @nodoc
class _$OsNotificationCopyWithImpl<$Res>
    implements $OsNotificationCopyWith<$Res> {
  _$OsNotificationCopyWithImpl(this._self, this._then);

  final OsNotification _self;
  final $Res Function(OsNotification) _then;

/// Create a copy of OsNotification
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? channel = null,Object? title = null,Object? body = null,Object? payload = freezed,Object? notifyAt = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,channel: null == channel ? _self.channel : channel // ignore: cast_nullable_to_non_nullable
as NotificationChannel,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,body: null == body ? _self.body : body // ignore: cast_nullable_to_non_nullable
as String,payload: freezed == payload ? _self.payload : payload // ignore: cast_nullable_to_non_nullable
as String?,notifyAt: null == notifyAt ? _self.notifyAt : notifyAt // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}

}


/// Adds pattern-matching-related methods to [OsNotification].
extension OsNotificationPatterns on OsNotification {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _OsNotification value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _OsNotification() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _OsNotification value)  $default,){
final _that = this;
switch (_that) {
case _OsNotification():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _OsNotification value)?  $default,){
final _that = this;
switch (_that) {
case _OsNotification() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int id,  NotificationChannel channel,  String title,  String body,  String? payload,  DateTime notifyAt)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _OsNotification() when $default != null:
return $default(_that.id,_that.channel,_that.title,_that.body,_that.payload,_that.notifyAt);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int id,  NotificationChannel channel,  String title,  String body,  String? payload,  DateTime notifyAt)  $default,) {final _that = this;
switch (_that) {
case _OsNotification():
return $default(_that.id,_that.channel,_that.title,_that.body,_that.payload,_that.notifyAt);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int id,  NotificationChannel channel,  String title,  String body,  String? payload,  DateTime notifyAt)?  $default,) {final _that = this;
switch (_that) {
case _OsNotification() when $default != null:
return $default(_that.id,_that.channel,_that.title,_that.body,_that.payload,_that.notifyAt);case _:
  return null;

}
}

}

/// @nodoc


class _OsNotification implements OsNotification {
  const _OsNotification({required this.id, required this.channel, required this.title, required this.body, this.payload, required this.notifyAt});
  

/// A unique identifier for this notification.
@override final  int id;
/// The channel through which this notification will be delivered.
///
/// Channels are used on modern Android versions to group notifications and
/// allow users to manage them.
@override final  NotificationChannel channel;
/// The localized title of the notification.
@override final  String title;
/// The localized body of the notification.
@override final  String body;
/// The payload of the notification.
@override final  String? payload;
/// The exact date and time when the notification should be displayed.
@override final  DateTime notifyAt;

/// Create a copy of OsNotification
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$OsNotificationCopyWith<_OsNotification> get copyWith => __$OsNotificationCopyWithImpl<_OsNotification>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _OsNotification&&(identical(other.id, id) || other.id == id)&&(identical(other.channel, channel) || other.channel == channel)&&(identical(other.title, title) || other.title == title)&&(identical(other.body, body) || other.body == body)&&(identical(other.payload, payload) || other.payload == payload)&&(identical(other.notifyAt, notifyAt) || other.notifyAt == notifyAt));
}


@override
int get hashCode => Object.hash(runtimeType,id,channel,title,body,payload,notifyAt);

@override
String toString() {
  return 'OsNotification(id: $id, channel: $channel, title: $title, body: $body, payload: $payload, notifyAt: $notifyAt)';
}


}

/// @nodoc
abstract mixin class _$OsNotificationCopyWith<$Res> implements $OsNotificationCopyWith<$Res> {
  factory _$OsNotificationCopyWith(_OsNotification value, $Res Function(_OsNotification) _then) = __$OsNotificationCopyWithImpl;
@override @useResult
$Res call({
 int id, NotificationChannel channel, String title, String body, String? payload, DateTime notifyAt
});




}
/// @nodoc
class __$OsNotificationCopyWithImpl<$Res>
    implements _$OsNotificationCopyWith<$Res> {
  __$OsNotificationCopyWithImpl(this._self, this._then);

  final _OsNotification _self;
  final $Res Function(_OsNotification) _then;

/// Create a copy of OsNotification
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? channel = null,Object? title = null,Object? body = null,Object? payload = freezed,Object? notifyAt = null,}) {
  return _then(_OsNotification(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,channel: null == channel ? _self.channel : channel // ignore: cast_nullable_to_non_nullable
as NotificationChannel,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,body: null == body ? _self.body : body // ignore: cast_nullable_to_non_nullable
as String,payload: freezed == payload ? _self.payload : payload // ignore: cast_nullable_to_non_nullable
as String?,notifyAt: null == notifyAt ? _self.notifyAt : notifyAt // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}


}

// dart format on
