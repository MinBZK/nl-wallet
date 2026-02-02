// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'wallet_banner.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$WalletBanner {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletBanner);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletBanner()';
}


}

/// @nodoc
class $WalletBannerCopyWith<$Res>  {
$WalletBannerCopyWith(WalletBanner _, $Res Function(WalletBanner) __);
}


/// Adds pattern-matching-related methods to [WalletBanner].
extension WalletBannerPatterns on WalletBanner {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( UpdateAvailableBanner value)?  updateAvailable,TResult Function( TourSuggestionBanner value)?  tourSuggestion,TResult Function( CardExpiresSoonBanner value)?  cardExpiresSoon,TResult Function( CardExpiredBanner value)?  cardExpired,TResult Function( CardRevokedBanner value)?  cardRevoked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case UpdateAvailableBanner() when updateAvailable != null:
return updateAvailable(_that);case TourSuggestionBanner() when tourSuggestion != null:
return tourSuggestion(_that);case CardExpiresSoonBanner() when cardExpiresSoon != null:
return cardExpiresSoon(_that);case CardExpiredBanner() when cardExpired != null:
return cardExpired(_that);case CardRevokedBanner() when cardRevoked != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( UpdateAvailableBanner value)  updateAvailable,required TResult Function( TourSuggestionBanner value)  tourSuggestion,required TResult Function( CardExpiresSoonBanner value)  cardExpiresSoon,required TResult Function( CardExpiredBanner value)  cardExpired,required TResult Function( CardRevokedBanner value)  cardRevoked,}){
final _that = this;
switch (_that) {
case UpdateAvailableBanner():
return updateAvailable(_that);case TourSuggestionBanner():
return tourSuggestion(_that);case CardExpiresSoonBanner():
return cardExpiresSoon(_that);case CardExpiredBanner():
return cardExpired(_that);case CardRevokedBanner():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( UpdateAvailableBanner value)?  updateAvailable,TResult? Function( TourSuggestionBanner value)?  tourSuggestion,TResult? Function( CardExpiresSoonBanner value)?  cardExpiresSoon,TResult? Function( CardExpiredBanner value)?  cardExpired,TResult? Function( CardRevokedBanner value)?  cardRevoked,}){
final _that = this;
switch (_that) {
case UpdateAvailableBanner() when updateAvailable != null:
return updateAvailable(_that);case TourSuggestionBanner() when tourSuggestion != null:
return tourSuggestion(_that);case CardExpiresSoonBanner() when cardExpiresSoon != null:
return cardExpiresSoon(_that);case CardExpiredBanner() when cardExpired != null:
return cardExpired(_that);case CardRevokedBanner() when cardRevoked != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( VersionState state)?  updateAvailable,TResult Function()?  tourSuggestion,TResult Function( WalletCard card,  DateTime expiresAt)?  cardExpiresSoon,TResult Function( WalletCard card)?  cardExpired,TResult Function( WalletCard card)?  cardRevoked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case UpdateAvailableBanner() when updateAvailable != null:
return updateAvailable(_that.state);case TourSuggestionBanner() when tourSuggestion != null:
return tourSuggestion();case CardExpiresSoonBanner() when cardExpiresSoon != null:
return cardExpiresSoon(_that.card,_that.expiresAt);case CardExpiredBanner() when cardExpired != null:
return cardExpired(_that.card);case CardRevokedBanner() when cardRevoked != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( VersionState state)  updateAvailable,required TResult Function()  tourSuggestion,required TResult Function( WalletCard card,  DateTime expiresAt)  cardExpiresSoon,required TResult Function( WalletCard card)  cardExpired,required TResult Function( WalletCard card)  cardRevoked,}) {final _that = this;
switch (_that) {
case UpdateAvailableBanner():
return updateAvailable(_that.state);case TourSuggestionBanner():
return tourSuggestion();case CardExpiresSoonBanner():
return cardExpiresSoon(_that.card,_that.expiresAt);case CardExpiredBanner():
return cardExpired(_that.card);case CardRevokedBanner():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( VersionState state)?  updateAvailable,TResult? Function()?  tourSuggestion,TResult? Function( WalletCard card,  DateTime expiresAt)?  cardExpiresSoon,TResult? Function( WalletCard card)?  cardExpired,TResult? Function( WalletCard card)?  cardRevoked,}) {final _that = this;
switch (_that) {
case UpdateAvailableBanner() when updateAvailable != null:
return updateAvailable(_that.state);case TourSuggestionBanner() when tourSuggestion != null:
return tourSuggestion();case CardExpiresSoonBanner() when cardExpiresSoon != null:
return cardExpiresSoon(_that.card,_that.expiresAt);case CardExpiredBanner() when cardExpired != null:
return cardExpired(_that.card);case CardRevokedBanner() when cardRevoked != null:
return cardRevoked(_that.card);case _:
  return null;

}
}

}

/// @nodoc


class UpdateAvailableBanner implements WalletBanner {
  const UpdateAvailableBanner({required this.state});
  

 final  VersionState state;

/// Create a copy of WalletBanner
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UpdateAvailableBannerCopyWith<UpdateAvailableBanner> get copyWith => _$UpdateAvailableBannerCopyWithImpl<UpdateAvailableBanner>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UpdateAvailableBanner&&(identical(other.state, state) || other.state == state));
}


@override
int get hashCode => Object.hash(runtimeType,state);

@override
String toString() {
  return 'WalletBanner.updateAvailable(state: $state)';
}


}

/// @nodoc
abstract mixin class $UpdateAvailableBannerCopyWith<$Res> implements $WalletBannerCopyWith<$Res> {
  factory $UpdateAvailableBannerCopyWith(UpdateAvailableBanner value, $Res Function(UpdateAvailableBanner) _then) = _$UpdateAvailableBannerCopyWithImpl;
@useResult
$Res call({
 VersionState state
});




}
/// @nodoc
class _$UpdateAvailableBannerCopyWithImpl<$Res>
    implements $UpdateAvailableBannerCopyWith<$Res> {
  _$UpdateAvailableBannerCopyWithImpl(this._self, this._then);

  final UpdateAvailableBanner _self;
  final $Res Function(UpdateAvailableBanner) _then;

/// Create a copy of WalletBanner
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? state = null,}) {
  return _then(UpdateAvailableBanner(
state: null == state ? _self.state : state // ignore: cast_nullable_to_non_nullable
as VersionState,
  ));
}


}

/// @nodoc


class TourSuggestionBanner implements WalletBanner {
  const TourSuggestionBanner();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TourSuggestionBanner);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletBanner.tourSuggestion()';
}


}




/// @nodoc


class CardExpiresSoonBanner implements WalletBanner {
  const CardExpiresSoonBanner({required this.card, required this.expiresAt});
  

 final  WalletCard card;
 final  DateTime expiresAt;

/// Create a copy of WalletBanner
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardExpiresSoonBannerCopyWith<CardExpiresSoonBanner> get copyWith => _$CardExpiresSoonBannerCopyWithImpl<CardExpiresSoonBanner>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardExpiresSoonBanner&&(identical(other.card, card) || other.card == card)&&(identical(other.expiresAt, expiresAt) || other.expiresAt == expiresAt));
}


@override
int get hashCode => Object.hash(runtimeType,card,expiresAt);

@override
String toString() {
  return 'WalletBanner.cardExpiresSoon(card: $card, expiresAt: $expiresAt)';
}


}

/// @nodoc
abstract mixin class $CardExpiresSoonBannerCopyWith<$Res> implements $WalletBannerCopyWith<$Res> {
  factory $CardExpiresSoonBannerCopyWith(CardExpiresSoonBanner value, $Res Function(CardExpiresSoonBanner) _then) = _$CardExpiresSoonBannerCopyWithImpl;
@useResult
$Res call({
 WalletCard card, DateTime expiresAt
});


$WalletCardCopyWith<$Res> get card;

}
/// @nodoc
class _$CardExpiresSoonBannerCopyWithImpl<$Res>
    implements $CardExpiresSoonBannerCopyWith<$Res> {
  _$CardExpiresSoonBannerCopyWithImpl(this._self, this._then);

  final CardExpiresSoonBanner _self;
  final $Res Function(CardExpiresSoonBanner) _then;

/// Create a copy of WalletBanner
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? card = null,Object? expiresAt = null,}) {
  return _then(CardExpiresSoonBanner(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as WalletCard,expiresAt: null == expiresAt ? _self.expiresAt : expiresAt // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}

/// Create a copy of WalletBanner
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


class CardExpiredBanner implements WalletBanner {
  const CardExpiredBanner({required this.card});
  

 final  WalletCard card;

/// Create a copy of WalletBanner
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardExpiredBannerCopyWith<CardExpiredBanner> get copyWith => _$CardExpiredBannerCopyWithImpl<CardExpiredBanner>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardExpiredBanner&&(identical(other.card, card) || other.card == card));
}


@override
int get hashCode => Object.hash(runtimeType,card);

@override
String toString() {
  return 'WalletBanner.cardExpired(card: $card)';
}


}

/// @nodoc
abstract mixin class $CardExpiredBannerCopyWith<$Res> implements $WalletBannerCopyWith<$Res> {
  factory $CardExpiredBannerCopyWith(CardExpiredBanner value, $Res Function(CardExpiredBanner) _then) = _$CardExpiredBannerCopyWithImpl;
@useResult
$Res call({
 WalletCard card
});


$WalletCardCopyWith<$Res> get card;

}
/// @nodoc
class _$CardExpiredBannerCopyWithImpl<$Res>
    implements $CardExpiredBannerCopyWith<$Res> {
  _$CardExpiredBannerCopyWithImpl(this._self, this._then);

  final CardExpiredBanner _self;
  final $Res Function(CardExpiredBanner) _then;

/// Create a copy of WalletBanner
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? card = null,}) {
  return _then(CardExpiredBanner(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as WalletCard,
  ));
}

/// Create a copy of WalletBanner
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


class CardRevokedBanner implements WalletBanner {
  const CardRevokedBanner({required this.card});
  

 final  WalletCard card;

/// Create a copy of WalletBanner
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardRevokedBannerCopyWith<CardRevokedBanner> get copyWith => _$CardRevokedBannerCopyWithImpl<CardRevokedBanner>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardRevokedBanner&&(identical(other.card, card) || other.card == card));
}


@override
int get hashCode => Object.hash(runtimeType,card);

@override
String toString() {
  return 'WalletBanner.cardRevoked(card: $card)';
}


}

/// @nodoc
abstract mixin class $CardRevokedBannerCopyWith<$Res> implements $WalletBannerCopyWith<$Res> {
  factory $CardRevokedBannerCopyWith(CardRevokedBanner value, $Res Function(CardRevokedBanner) _then) = _$CardRevokedBannerCopyWithImpl;
@useResult
$Res call({
 WalletCard card
});


$WalletCardCopyWith<$Res> get card;

}
/// @nodoc
class _$CardRevokedBannerCopyWithImpl<$Res>
    implements $CardRevokedBannerCopyWith<$Res> {
  _$CardRevokedBannerCopyWithImpl(this._self, this._then);

  final CardRevokedBanner _self;
  final $Res Function(CardRevokedBanner) _then;

/// Create a copy of WalletBanner
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? card = null,}) {
  return _then(CardRevokedBanner(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as WalletCard,
  ));
}

/// Create a copy of WalletBanner
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
