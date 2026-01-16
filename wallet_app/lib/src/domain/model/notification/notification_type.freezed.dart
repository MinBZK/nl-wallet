// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'notification_type.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NotificationType {

 WalletCard get card;
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
 WalletCard card
});


$WalletCardCopyWith<$Res> get card;

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
as WalletCard,
  ));
}
/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WalletCardCopyWith<$Res> get card {
  
  return $WalletCardCopyWith<$Res>(_self.card, (value) {
    return _then(_self.copyWith(card: value));
  });
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CardExpiresSoon value)?  cardExpiresSoon,TResult Function( CardExpired value)?  cardExpired,TResult Function( CardRevoked value)?  cardRevoked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CardExpiresSoon() when cardExpiresSoon != null:
return cardExpiresSoon(_that);case CardExpired() when cardExpired != null:
return cardExpired(_that);case CardRevoked() when cardRevoked != null:
return cardRevoked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CardExpiresSoon value)  cardExpiresSoon,required TResult Function( CardExpired value)  cardExpired,required TResult Function( CardRevoked value)  cardRevoked,}){
final _that = this;
switch (_that) {
case CardExpiresSoon():
return cardExpiresSoon(_that);case CardExpired():
return cardExpired(_that);case CardRevoked():
return cardRevoked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CardExpiresSoon value)?  cardExpiresSoon,TResult? Function( CardExpired value)?  cardExpired,TResult? Function( CardRevoked value)?  cardRevoked,}){
final _that = this;
switch (_that) {
case CardExpiresSoon() when cardExpiresSoon != null:
return cardExpiresSoon(_that);case CardExpired() when cardExpired != null:
return cardExpired(_that);case CardRevoked() when cardRevoked != null:
return cardRevoked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WalletCard card,  DateTime expiresAt)?  cardExpiresSoon,TResult Function( WalletCard card)?  cardExpired,TResult Function( WalletCard card)?  cardRevoked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CardExpiresSoon() when cardExpiresSoon != null:
return cardExpiresSoon(_that.card,_that.expiresAt);case CardExpired() when cardExpired != null:
return cardExpired(_that.card);case CardRevoked() when cardRevoked != null:
return cardRevoked(_that.card);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WalletCard card,  DateTime expiresAt)  cardExpiresSoon,required TResult Function( WalletCard card)  cardExpired,required TResult Function( WalletCard card)  cardRevoked,}) {final _that = this;
switch (_that) {
case CardExpiresSoon():
return cardExpiresSoon(_that.card,_that.expiresAt);case CardExpired():
return cardExpired(_that.card);case CardRevoked():
return cardRevoked(_that.card);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WalletCard card,  DateTime expiresAt)?  cardExpiresSoon,TResult? Function( WalletCard card)?  cardExpired,TResult? Function( WalletCard card)?  cardRevoked,}) {final _that = this;
switch (_that) {
case CardExpiresSoon() when cardExpiresSoon != null:
return cardExpiresSoon(_that.card,_that.expiresAt);case CardExpired() when cardExpired != null:
return cardExpired(_that.card);case CardRevoked() when cardRevoked != null:
return cardRevoked(_that.card);case _:
  return null;

}
}

}

/// @nodoc


class CardExpiresSoon implements NotificationType {
  const CardExpiresSoon({required this.card, required this.expiresAt});
  

@override final  WalletCard card;
 final  DateTime expiresAt;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardExpiresSoonCopyWith<CardExpiresSoon> get copyWith => _$CardExpiresSoonCopyWithImpl<CardExpiresSoon>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardExpiresSoon&&(identical(other.card, card) || other.card == card)&&(identical(other.expiresAt, expiresAt) || other.expiresAt == expiresAt));
}


@override
int get hashCode => Object.hash(runtimeType,card,expiresAt);

@override
String toString() {
  return 'NotificationType.cardExpiresSoon(card: $card, expiresAt: $expiresAt)';
}


}

/// @nodoc
abstract mixin class $CardExpiresSoonCopyWith<$Res> implements $NotificationTypeCopyWith<$Res> {
  factory $CardExpiresSoonCopyWith(CardExpiresSoon value, $Res Function(CardExpiresSoon) _then) = _$CardExpiresSoonCopyWithImpl;
@override @useResult
$Res call({
 WalletCard card, DateTime expiresAt
});


@override $WalletCardCopyWith<$Res> get card;

}
/// @nodoc
class _$CardExpiresSoonCopyWithImpl<$Res>
    implements $CardExpiresSoonCopyWith<$Res> {
  _$CardExpiresSoonCopyWithImpl(this._self, this._then);

  final CardExpiresSoon _self;
  final $Res Function(CardExpiresSoon) _then;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? card = null,Object? expiresAt = null,}) {
  return _then(CardExpiresSoon(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as WalletCard,expiresAt: null == expiresAt ? _self.expiresAt : expiresAt // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WalletCardCopyWith<$Res> get card {
  
  return $WalletCardCopyWith<$Res>(_self.card, (value) {
    return _then(_self.copyWith(card: value));
  });
}
}

/// @nodoc


class CardExpired implements NotificationType {
  const CardExpired({required this.card});
  

@override final  WalletCard card;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardExpiredCopyWith<CardExpired> get copyWith => _$CardExpiredCopyWithImpl<CardExpired>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardExpired&&(identical(other.card, card) || other.card == card));
}


@override
int get hashCode => Object.hash(runtimeType,card);

@override
String toString() {
  return 'NotificationType.cardExpired(card: $card)';
}


}

/// @nodoc
abstract mixin class $CardExpiredCopyWith<$Res> implements $NotificationTypeCopyWith<$Res> {
  factory $CardExpiredCopyWith(CardExpired value, $Res Function(CardExpired) _then) = _$CardExpiredCopyWithImpl;
@override @useResult
$Res call({
 WalletCard card
});


@override $WalletCardCopyWith<$Res> get card;

}
/// @nodoc
class _$CardExpiredCopyWithImpl<$Res>
    implements $CardExpiredCopyWith<$Res> {
  _$CardExpiredCopyWithImpl(this._self, this._then);

  final CardExpired _self;
  final $Res Function(CardExpired) _then;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? card = null,}) {
  return _then(CardExpired(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as WalletCard,
  ));
}

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WalletCardCopyWith<$Res> get card {
  
  return $WalletCardCopyWith<$Res>(_self.card, (value) {
    return _then(_self.copyWith(card: value));
  });
}
}

/// @nodoc


class CardRevoked implements NotificationType {
  const CardRevoked({required this.card});
  

@override final  WalletCard card;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardRevokedCopyWith<CardRevoked> get copyWith => _$CardRevokedCopyWithImpl<CardRevoked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardRevoked&&(identical(other.card, card) || other.card == card));
}


@override
int get hashCode => Object.hash(runtimeType,card);

@override
String toString() {
  return 'NotificationType.cardRevoked(card: $card)';
}


}

/// @nodoc
abstract mixin class $CardRevokedCopyWith<$Res> implements $NotificationTypeCopyWith<$Res> {
  factory $CardRevokedCopyWith(CardRevoked value, $Res Function(CardRevoked) _then) = _$CardRevokedCopyWithImpl;
@override @useResult
$Res call({
 WalletCard card
});


@override $WalletCardCopyWith<$Res> get card;

}
/// @nodoc
class _$CardRevokedCopyWithImpl<$Res>
    implements $CardRevokedCopyWith<$Res> {
  _$CardRevokedCopyWithImpl(this._self, this._then);

  final CardRevoked _self;
  final $Res Function(CardRevoked) _then;

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? card = null,}) {
  return _then(CardRevoked(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as WalletCard,
  ));
}

/// Create a copy of NotificationType
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WalletCardCopyWith<$Res> get card {
  
  return $WalletCardCopyWith<$Res>(_self.card, (value) {
    return _then(_self.copyWith(card: value));
  });
}
}

// dart format on
