// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'notification.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DisplayTarget {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DisplayTarget);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DisplayTarget()';
}


}

/// @nodoc
class $DisplayTargetCopyWith<$Res>  {
$DisplayTargetCopyWith(DisplayTarget _, $Res Function(DisplayTarget) __);
}


/// Adds pattern-matching-related methods to [DisplayTarget].
extension DisplayTargetPatterns on DisplayTarget {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DisplayTarget_Os value)?  os,TResult Function( DisplayTarget_Dashboard value)?  dashboard,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DisplayTarget_Os() when os != null:
return os(_that);case DisplayTarget_Dashboard() when dashboard != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DisplayTarget_Os value)  os,required TResult Function( DisplayTarget_Dashboard value)  dashboard,}){
final _that = this;
switch (_that) {
case DisplayTarget_Os():
return os(_that);case DisplayTarget_Dashboard():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DisplayTarget_Os value)?  os,TResult? Function( DisplayTarget_Dashboard value)?  dashboard,}){
final _that = this;
switch (_that) {
case DisplayTarget_Os() when os != null:
return os(_that);case DisplayTarget_Dashboard() when dashboard != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( NotifyAt notifyAt)?  os,TResult Function()?  dashboard,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DisplayTarget_Os() when os != null:
return os(_that.notifyAt);case DisplayTarget_Dashboard() when dashboard != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( NotifyAt notifyAt)  os,required TResult Function()  dashboard,}) {final _that = this;
switch (_that) {
case DisplayTarget_Os():
return os(_that.notifyAt);case DisplayTarget_Dashboard():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( NotifyAt notifyAt)?  os,TResult? Function()?  dashboard,}) {final _that = this;
switch (_that) {
case DisplayTarget_Os() when os != null:
return os(_that.notifyAt);case DisplayTarget_Dashboard() when dashboard != null:
return dashboard();case _:
  return null;

}
}

}

/// @nodoc


class DisplayTarget_Os extends DisplayTarget {
  const DisplayTarget_Os({required this.notifyAt}): super._();
  

 final  NotifyAt notifyAt;

/// Create a copy of DisplayTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DisplayTarget_OsCopyWith<DisplayTarget_Os> get copyWith => _$DisplayTarget_OsCopyWithImpl<DisplayTarget_Os>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DisplayTarget_Os&&(identical(other.notifyAt, notifyAt) || other.notifyAt == notifyAt));
}


@override
int get hashCode => Object.hash(runtimeType,notifyAt);

@override
String toString() {
  return 'DisplayTarget.os(notifyAt: $notifyAt)';
}


}

/// @nodoc
abstract mixin class $DisplayTarget_OsCopyWith<$Res> implements $DisplayTargetCopyWith<$Res> {
  factory $DisplayTarget_OsCopyWith(DisplayTarget_Os value, $Res Function(DisplayTarget_Os) _then) = _$DisplayTarget_OsCopyWithImpl;
@useResult
$Res call({
 NotifyAt notifyAt
});


$NotifyAtCopyWith<$Res> get notifyAt;

}
/// @nodoc
class _$DisplayTarget_OsCopyWithImpl<$Res>
    implements $DisplayTarget_OsCopyWith<$Res> {
  _$DisplayTarget_OsCopyWithImpl(this._self, this._then);

  final DisplayTarget_Os _self;
  final $Res Function(DisplayTarget_Os) _then;

/// Create a copy of DisplayTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notifyAt = null,}) {
  return _then(DisplayTarget_Os(
notifyAt: null == notifyAt ? _self.notifyAt : notifyAt // ignore: cast_nullable_to_non_nullable
as NotifyAt,
  ));
}

/// Create a copy of DisplayTarget
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NotifyAtCopyWith<$Res> get notifyAt {
  
  return $NotifyAtCopyWith<$Res>(_self.notifyAt, (value) {
    return _then(_self.copyWith(notifyAt: value));
  });
}
}

/// @nodoc


class DisplayTarget_Dashboard extends DisplayTarget {
  const DisplayTarget_Dashboard(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DisplayTarget_Dashboard);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DisplayTarget.dashboard()';
}


}




/// @nodoc
mixin _$NotificationType {

 AttestationPresentation get card;
/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotificationTypeCopyWith<NotificationType> get copyWith => _$NotificationTypeCopyWithImpl<NotificationType>(this as NotificationType, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotificationType&&(identical(other.card, card) || other.card == card));
}


@override
int get hashCode => Object.hash(runtimeType,card);

@override
String toString() {
  return 'NotificationType(card: $card)';
}


}

/// @nodoc
abstract mixin class $NotificationTypeCopyWith<$Res>  {
  factory $NotificationTypeCopyWith(NotificationType value, $Res Function(NotificationType) _then) = _$NotificationTypeCopyWithImpl;
@useResult
$Res call({
 AttestationPresentation card
});




}
/// @nodoc
class _$NotificationTypeCopyWithImpl<$Res>
    implements $NotificationTypeCopyWith<$Res> {
  _$NotificationTypeCopyWithImpl(this._self, this._then);

  final NotificationType _self;
  final $Res Function(NotificationType) _then;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? card = null,}) {
  return _then(_self.copyWith(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as AttestationPresentation,
  ));
}

}


/// Adds pattern-matching-related methods to [NotificationType].
extension NotificationTypePatterns on NotificationType {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NotificationType_CardExpired value)?  cardExpired,TResult Function( NotificationType_CardExpiresSoon value)?  cardExpiresSoon,TResult Function( NotificationType_Revoked value)?  revoked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NotificationType_CardExpired() when cardExpired != null:
return cardExpired(_that);case NotificationType_CardExpiresSoon() when cardExpiresSoon != null:
return cardExpiresSoon(_that);case NotificationType_Revoked() when revoked != null:
return revoked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NotificationType_CardExpired value)  cardExpired,required TResult Function( NotificationType_CardExpiresSoon value)  cardExpiresSoon,required TResult Function( NotificationType_Revoked value)  revoked,}){
final _that = this;
switch (_that) {
case NotificationType_CardExpired():
return cardExpired(_that);case NotificationType_CardExpiresSoon():
return cardExpiresSoon(_that);case NotificationType_Revoked():
return revoked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NotificationType_CardExpired value)?  cardExpired,TResult? Function( NotificationType_CardExpiresSoon value)?  cardExpiresSoon,TResult? Function( NotificationType_Revoked value)?  revoked,}){
final _that = this;
switch (_that) {
case NotificationType_CardExpired() when cardExpired != null:
return cardExpired(_that);case NotificationType_CardExpiresSoon() when cardExpiresSoon != null:
return cardExpiresSoon(_that);case NotificationType_Revoked() when revoked != null:
return revoked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( AttestationPresentation card)?  cardExpired,TResult Function( AttestationPresentation card,  String expiresAt)?  cardExpiresSoon,TResult Function( AttestationPresentation card)?  revoked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NotificationType_CardExpired() when cardExpired != null:
return cardExpired(_that.card);case NotificationType_CardExpiresSoon() when cardExpiresSoon != null:
return cardExpiresSoon(_that.card,_that.expiresAt);case NotificationType_Revoked() when revoked != null:
return revoked(_that.card);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( AttestationPresentation card)  cardExpired,required TResult Function( AttestationPresentation card,  String expiresAt)  cardExpiresSoon,required TResult Function( AttestationPresentation card)  revoked,}) {final _that = this;
switch (_that) {
case NotificationType_CardExpired():
return cardExpired(_that.card);case NotificationType_CardExpiresSoon():
return cardExpiresSoon(_that.card,_that.expiresAt);case NotificationType_Revoked():
return revoked(_that.card);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( AttestationPresentation card)?  cardExpired,TResult? Function( AttestationPresentation card,  String expiresAt)?  cardExpiresSoon,TResult? Function( AttestationPresentation card)?  revoked,}) {final _that = this;
switch (_that) {
case NotificationType_CardExpired() when cardExpired != null:
return cardExpired(_that.card);case NotificationType_CardExpiresSoon() when cardExpiresSoon != null:
return cardExpiresSoon(_that.card,_that.expiresAt);case NotificationType_Revoked() when revoked != null:
return revoked(_that.card);case _:
  return null;

}
}

}

/// @nodoc


class NotificationType_CardExpired extends NotificationType {
  const NotificationType_CardExpired({required this.card}): super._();
  

@override final  AttestationPresentation card;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotificationType_CardExpiredCopyWith<NotificationType_CardExpired> get copyWith => _$NotificationType_CardExpiredCopyWithImpl<NotificationType_CardExpired>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotificationType_CardExpired&&(identical(other.card, card) || other.card == card));
}


@override
int get hashCode => Object.hash(runtimeType,card);

@override
String toString() {
  return 'NotificationType.cardExpired(card: $card)';
}


}

/// @nodoc
abstract mixin class $NotificationType_CardExpiredCopyWith<$Res> implements $NotificationTypeCopyWith<$Res> {
  factory $NotificationType_CardExpiredCopyWith(NotificationType_CardExpired value, $Res Function(NotificationType_CardExpired) _then) = _$NotificationType_CardExpiredCopyWithImpl;
@override @useResult
$Res call({
 AttestationPresentation card
});




}
/// @nodoc
class _$NotificationType_CardExpiredCopyWithImpl<$Res>
    implements $NotificationType_CardExpiredCopyWith<$Res> {
  _$NotificationType_CardExpiredCopyWithImpl(this._self, this._then);

  final NotificationType_CardExpired _self;
  final $Res Function(NotificationType_CardExpired) _then;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? card = null,}) {
  return _then(NotificationType_CardExpired(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as AttestationPresentation,
  ));
}


}

/// @nodoc


class NotificationType_CardExpiresSoon extends NotificationType {
  const NotificationType_CardExpiresSoon({required this.card, required this.expiresAt}): super._();
  

@override final  AttestationPresentation card;
 final  String expiresAt;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotificationType_CardExpiresSoonCopyWith<NotificationType_CardExpiresSoon> get copyWith => _$NotificationType_CardExpiresSoonCopyWithImpl<NotificationType_CardExpiresSoon>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotificationType_CardExpiresSoon&&(identical(other.card, card) || other.card == card)&&(identical(other.expiresAt, expiresAt) || other.expiresAt == expiresAt));
}


@override
int get hashCode => Object.hash(runtimeType,card,expiresAt);

@override
String toString() {
  return 'NotificationType.cardExpiresSoon(card: $card, expiresAt: $expiresAt)';
}


}

/// @nodoc
abstract mixin class $NotificationType_CardExpiresSoonCopyWith<$Res> implements $NotificationTypeCopyWith<$Res> {
  factory $NotificationType_CardExpiresSoonCopyWith(NotificationType_CardExpiresSoon value, $Res Function(NotificationType_CardExpiresSoon) _then) = _$NotificationType_CardExpiresSoonCopyWithImpl;
@override @useResult
$Res call({
 AttestationPresentation card, String expiresAt
});




}
/// @nodoc
class _$NotificationType_CardExpiresSoonCopyWithImpl<$Res>
    implements $NotificationType_CardExpiresSoonCopyWith<$Res> {
  _$NotificationType_CardExpiresSoonCopyWithImpl(this._self, this._then);

  final NotificationType_CardExpiresSoon _self;
  final $Res Function(NotificationType_CardExpiresSoon) _then;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? card = null,Object? expiresAt = null,}) {
  return _then(NotificationType_CardExpiresSoon(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as AttestationPresentation,expiresAt: null == expiresAt ? _self.expiresAt : expiresAt // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotificationType_Revoked extends NotificationType {
  const NotificationType_Revoked({required this.card}): super._();
  

@override final  AttestationPresentation card;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotificationType_RevokedCopyWith<NotificationType_Revoked> get copyWith => _$NotificationType_RevokedCopyWithImpl<NotificationType_Revoked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotificationType_Revoked&&(identical(other.card, card) || other.card == card));
}


@override
int get hashCode => Object.hash(runtimeType,card);

@override
String toString() {
  return 'NotificationType.revoked(card: $card)';
}


}

/// @nodoc
abstract mixin class $NotificationType_RevokedCopyWith<$Res> implements $NotificationTypeCopyWith<$Res> {
  factory $NotificationType_RevokedCopyWith(NotificationType_Revoked value, $Res Function(NotificationType_Revoked) _then) = _$NotificationType_RevokedCopyWithImpl;
@override @useResult
$Res call({
 AttestationPresentation card
});




}
/// @nodoc
class _$NotificationType_RevokedCopyWithImpl<$Res>
    implements $NotificationType_RevokedCopyWith<$Res> {
  _$NotificationType_RevokedCopyWithImpl(this._self, this._then);

  final NotificationType_Revoked _self;
  final $Res Function(NotificationType_Revoked) _then;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? card = null,}) {
  return _then(NotificationType_Revoked(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as AttestationPresentation,
  ));
}


}

/// @nodoc
mixin _$NotifyAt {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotifyAt);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotifyAt()';
}


}

/// @nodoc
class $NotifyAtCopyWith<$Res>  {
$NotifyAtCopyWith(NotifyAt _, $Res Function(NotifyAt) __);
}


/// Adds pattern-matching-related methods to [NotifyAt].
extension NotifyAtPatterns on NotifyAt {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NotifyAt_Now value)?  now,TResult Function( NotifyAt_At value)?  at,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NotifyAt_Now() when now != null:
return now(_that);case NotifyAt_At() when at != null:
return at(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NotifyAt_Now value)  now,required TResult Function( NotifyAt_At value)  at,}){
final _that = this;
switch (_that) {
case NotifyAt_Now():
return now(_that);case NotifyAt_At():
return at(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NotifyAt_Now value)?  now,TResult? Function( NotifyAt_At value)?  at,}){
final _that = this;
switch (_that) {
case NotifyAt_Now() when now != null:
return now(_that);case NotifyAt_At() when at != null:
return at(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  now,TResult Function( String field0)?  at,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NotifyAt_Now() when now != null:
return now();case NotifyAt_At() when at != null:
return at(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  now,required TResult Function( String field0)  at,}) {final _that = this;
switch (_that) {
case NotifyAt_Now():
return now();case NotifyAt_At():
return at(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  now,TResult? Function( String field0)?  at,}) {final _that = this;
switch (_that) {
case NotifyAt_Now() when now != null:
return now();case NotifyAt_At() when at != null:
return at(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class NotifyAt_Now extends NotifyAt {
  const NotifyAt_Now(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotifyAt_Now);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotifyAt.now()';
}


}




/// @nodoc


class NotifyAt_At extends NotifyAt {
  const NotifyAt_At(this.field0): super._();
  

 final  String field0;

/// Create a copy of NotifyAt
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotifyAt_AtCopyWith<NotifyAt_At> get copyWith => _$NotifyAt_AtCopyWithImpl<NotifyAt_At>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotifyAt_At&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'NotifyAt.at(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $NotifyAt_AtCopyWith<$Res> implements $NotifyAtCopyWith<$Res> {
  factory $NotifyAt_AtCopyWith(NotifyAt_At value, $Res Function(NotifyAt_At) _then) = _$NotifyAt_AtCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$NotifyAt_AtCopyWithImpl<$Res>
    implements $NotifyAt_AtCopyWith<$Res> {
  _$NotifyAt_AtCopyWithImpl(this._self, this._then);

  final NotifyAt_At _self;
  final $Res Function(NotifyAt_At) _then;

/// Create a copy of NotifyAt
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(NotifyAt_At(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
