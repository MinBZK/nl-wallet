// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'card_status.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
CardStatus _$CardStatusFromJson(
  Map<String, dynamic> json
) {
        switch (json['runtimeType']) {
                  case 'validSoon':
          return CardStatusValidSoon.fromJson(
            json
          );
                case 'valid':
          return CardStatusValid.fromJson(
            json
          );
                case 'expiresSoon':
          return CardStatusExpiresSoon.fromJson(
            json
          );
                case 'expired':
          return CardStatusExpired.fromJson(
            json
          );
                case 'revoked':
          return CardStatusRevoked.fromJson(
            json
          );
                case 'corrupted':
          return CardStatusCorrupted.fromJson(
            json
          );
                case 'undetermined':
          return CardStatusUndetermined.fromJson(
            json
          );
        
          default:
            throw CheckedFromJsonException(
  json,
  'runtimeType',
  'CardStatus',
  'Invalid union type "${json['runtimeType']}"!'
);
        }
      
}

/// @nodoc
mixin _$CardStatus {



  /// Serializes this CardStatus to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardStatus);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CardStatus()';
}


}

/// @nodoc
class $CardStatusCopyWith<$Res>  {
$CardStatusCopyWith(CardStatus _, $Res Function(CardStatus) __);
}


/// Adds pattern-matching-related methods to [CardStatus].
extension CardStatusPatterns on CardStatus {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CardStatusValidSoon value)?  validSoon,TResult Function( CardStatusValid value)?  valid,TResult Function( CardStatusExpiresSoon value)?  expiresSoon,TResult Function( CardStatusExpired value)?  expired,TResult Function( CardStatusRevoked value)?  revoked,TResult Function( CardStatusCorrupted value)?  corrupted,TResult Function( CardStatusUndetermined value)?  undetermined,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CardStatusValidSoon() when validSoon != null:
return validSoon(_that);case CardStatusValid() when valid != null:
return valid(_that);case CardStatusExpiresSoon() when expiresSoon != null:
return expiresSoon(_that);case CardStatusExpired() when expired != null:
return expired(_that);case CardStatusRevoked() when revoked != null:
return revoked(_that);case CardStatusCorrupted() when corrupted != null:
return corrupted(_that);case CardStatusUndetermined() when undetermined != null:
return undetermined(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CardStatusValidSoon value)  validSoon,required TResult Function( CardStatusValid value)  valid,required TResult Function( CardStatusExpiresSoon value)  expiresSoon,required TResult Function( CardStatusExpired value)  expired,required TResult Function( CardStatusRevoked value)  revoked,required TResult Function( CardStatusCorrupted value)  corrupted,required TResult Function( CardStatusUndetermined value)  undetermined,}){
final _that = this;
switch (_that) {
case CardStatusValidSoon():
return validSoon(_that);case CardStatusValid():
return valid(_that);case CardStatusExpiresSoon():
return expiresSoon(_that);case CardStatusExpired():
return expired(_that);case CardStatusRevoked():
return revoked(_that);case CardStatusCorrupted():
return corrupted(_that);case CardStatusUndetermined():
return undetermined(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CardStatusValidSoon value)?  validSoon,TResult? Function( CardStatusValid value)?  valid,TResult? Function( CardStatusExpiresSoon value)?  expiresSoon,TResult? Function( CardStatusExpired value)?  expired,TResult? Function( CardStatusRevoked value)?  revoked,TResult? Function( CardStatusCorrupted value)?  corrupted,TResult? Function( CardStatusUndetermined value)?  undetermined,}){
final _that = this;
switch (_that) {
case CardStatusValidSoon() when validSoon != null:
return validSoon(_that);case CardStatusValid() when valid != null:
return valid(_that);case CardStatusExpiresSoon() when expiresSoon != null:
return expiresSoon(_that);case CardStatusExpired() when expired != null:
return expired(_that);case CardStatusRevoked() when revoked != null:
return revoked(_that);case CardStatusCorrupted() when corrupted != null:
return corrupted(_that);case CardStatusUndetermined() when undetermined != null:
return undetermined(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( DateTime validFrom)?  validSoon,TResult Function( DateTime? validUntil)?  valid,TResult Function( DateTime validUntil)?  expiresSoon,TResult Function( DateTime validUntil)?  expired,TResult Function()?  revoked,TResult Function()?  corrupted,TResult Function()?  undetermined,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CardStatusValidSoon() when validSoon != null:
return validSoon(_that.validFrom);case CardStatusValid() when valid != null:
return valid(_that.validUntil);case CardStatusExpiresSoon() when expiresSoon != null:
return expiresSoon(_that.validUntil);case CardStatusExpired() when expired != null:
return expired(_that.validUntil);case CardStatusRevoked() when revoked != null:
return revoked();case CardStatusCorrupted() when corrupted != null:
return corrupted();case CardStatusUndetermined() when undetermined != null:
return undetermined();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( DateTime validFrom)  validSoon,required TResult Function( DateTime? validUntil)  valid,required TResult Function( DateTime validUntil)  expiresSoon,required TResult Function( DateTime validUntil)  expired,required TResult Function()  revoked,required TResult Function()  corrupted,required TResult Function()  undetermined,}) {final _that = this;
switch (_that) {
case CardStatusValidSoon():
return validSoon(_that.validFrom);case CardStatusValid():
return valid(_that.validUntil);case CardStatusExpiresSoon():
return expiresSoon(_that.validUntil);case CardStatusExpired():
return expired(_that.validUntil);case CardStatusRevoked():
return revoked();case CardStatusCorrupted():
return corrupted();case CardStatusUndetermined():
return undetermined();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( DateTime validFrom)?  validSoon,TResult? Function( DateTime? validUntil)?  valid,TResult? Function( DateTime validUntil)?  expiresSoon,TResult? Function( DateTime validUntil)?  expired,TResult? Function()?  revoked,TResult? Function()?  corrupted,TResult? Function()?  undetermined,}) {final _that = this;
switch (_that) {
case CardStatusValidSoon() when validSoon != null:
return validSoon(_that.validFrom);case CardStatusValid() when valid != null:
return valid(_that.validUntil);case CardStatusExpiresSoon() when expiresSoon != null:
return expiresSoon(_that.validUntil);case CardStatusExpired() when expired != null:
return expired(_that.validUntil);case CardStatusRevoked() when revoked != null:
return revoked();case CardStatusCorrupted() when corrupted != null:
return corrupted();case CardStatusUndetermined() when undetermined != null:
return undetermined();case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class CardStatusValidSoon implements CardStatus {
  const CardStatusValidSoon({required this.validFrom, final  String? $type}): $type = $type ?? 'validSoon';
  factory CardStatusValidSoon.fromJson(Map<String, dynamic> json) => _$CardStatusValidSoonFromJson(json);

/// Time from which the card is valid
 final  DateTime validFrom;

@JsonKey(name: 'runtimeType')
final String $type;


/// Create a copy of CardStatus
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardStatusValidSoonCopyWith<CardStatusValidSoon> get copyWith => _$CardStatusValidSoonCopyWithImpl<CardStatusValidSoon>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$CardStatusValidSoonToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardStatusValidSoon&&(identical(other.validFrom, validFrom) || other.validFrom == validFrom));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,validFrom);

@override
String toString() {
  return 'CardStatus.validSoon(validFrom: $validFrom)';
}


}

/// @nodoc
abstract mixin class $CardStatusValidSoonCopyWith<$Res> implements $CardStatusCopyWith<$Res> {
  factory $CardStatusValidSoonCopyWith(CardStatusValidSoon value, $Res Function(CardStatusValidSoon) _then) = _$CardStatusValidSoonCopyWithImpl;
@useResult
$Res call({
 DateTime validFrom
});




}
/// @nodoc
class _$CardStatusValidSoonCopyWithImpl<$Res>
    implements $CardStatusValidSoonCopyWith<$Res> {
  _$CardStatusValidSoonCopyWithImpl(this._self, this._then);

  final CardStatusValidSoon _self;
  final $Res Function(CardStatusValidSoon) _then;

/// Create a copy of CardStatus
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? validFrom = null,}) {
  return _then(CardStatusValidSoon(
validFrom: null == validFrom ? _self.validFrom : validFrom // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}


}

/// @nodoc
@JsonSerializable()

class CardStatusValid implements CardStatus {
  const CardStatusValid({required this.validUntil, final  String? $type}): $type = $type ?? 'valid';
  factory CardStatusValid.fromJson(Map<String, dynamic> json) => _$CardStatusValidFromJson(json);

/// Time until the card is valid (expiry date)
 final  DateTime? validUntil;

@JsonKey(name: 'runtimeType')
final String $type;


/// Create a copy of CardStatus
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardStatusValidCopyWith<CardStatusValid> get copyWith => _$CardStatusValidCopyWithImpl<CardStatusValid>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$CardStatusValidToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardStatusValid&&(identical(other.validUntil, validUntil) || other.validUntil == validUntil));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,validUntil);

@override
String toString() {
  return 'CardStatus.valid(validUntil: $validUntil)';
}


}

/// @nodoc
abstract mixin class $CardStatusValidCopyWith<$Res> implements $CardStatusCopyWith<$Res> {
  factory $CardStatusValidCopyWith(CardStatusValid value, $Res Function(CardStatusValid) _then) = _$CardStatusValidCopyWithImpl;
@useResult
$Res call({
 DateTime? validUntil
});




}
/// @nodoc
class _$CardStatusValidCopyWithImpl<$Res>
    implements $CardStatusValidCopyWith<$Res> {
  _$CardStatusValidCopyWithImpl(this._self, this._then);

  final CardStatusValid _self;
  final $Res Function(CardStatusValid) _then;

/// Create a copy of CardStatus
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? validUntil = freezed,}) {
  return _then(CardStatusValid(
validUntil: freezed == validUntil ? _self.validUntil : validUntil // ignore: cast_nullable_to_non_nullable
as DateTime?,
  ));
}


}

/// @nodoc
@JsonSerializable()

class CardStatusExpiresSoon implements CardStatus {
  const CardStatusExpiresSoon({required this.validUntil, final  String? $type}): $type = $type ?? 'expiresSoon';
  factory CardStatusExpiresSoon.fromJson(Map<String, dynamic> json) => _$CardStatusExpiresSoonFromJson(json);

/// Time until the card is valid (expiry date)
 final  DateTime validUntil;

@JsonKey(name: 'runtimeType')
final String $type;


/// Create a copy of CardStatus
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardStatusExpiresSoonCopyWith<CardStatusExpiresSoon> get copyWith => _$CardStatusExpiresSoonCopyWithImpl<CardStatusExpiresSoon>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$CardStatusExpiresSoonToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardStatusExpiresSoon&&(identical(other.validUntil, validUntil) || other.validUntil == validUntil));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,validUntil);

@override
String toString() {
  return 'CardStatus.expiresSoon(validUntil: $validUntil)';
}


}

/// @nodoc
abstract mixin class $CardStatusExpiresSoonCopyWith<$Res> implements $CardStatusCopyWith<$Res> {
  factory $CardStatusExpiresSoonCopyWith(CardStatusExpiresSoon value, $Res Function(CardStatusExpiresSoon) _then) = _$CardStatusExpiresSoonCopyWithImpl;
@useResult
$Res call({
 DateTime validUntil
});




}
/// @nodoc
class _$CardStatusExpiresSoonCopyWithImpl<$Res>
    implements $CardStatusExpiresSoonCopyWith<$Res> {
  _$CardStatusExpiresSoonCopyWithImpl(this._self, this._then);

  final CardStatusExpiresSoon _self;
  final $Res Function(CardStatusExpiresSoon) _then;

/// Create a copy of CardStatus
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? validUntil = null,}) {
  return _then(CardStatusExpiresSoon(
validUntil: null == validUntil ? _self.validUntil : validUntil // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}


}

/// @nodoc
@JsonSerializable()

class CardStatusExpired implements CardStatus {
  const CardStatusExpired({required this.validUntil, final  String? $type}): $type = $type ?? 'expired';
  factory CardStatusExpired.fromJson(Map<String, dynamic> json) => _$CardStatusExpiredFromJson(json);

/// Time until the card is valid (expiry date)
 final  DateTime validUntil;

@JsonKey(name: 'runtimeType')
final String $type;


/// Create a copy of CardStatus
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardStatusExpiredCopyWith<CardStatusExpired> get copyWith => _$CardStatusExpiredCopyWithImpl<CardStatusExpired>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$CardStatusExpiredToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardStatusExpired&&(identical(other.validUntil, validUntil) || other.validUntil == validUntil));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,validUntil);

@override
String toString() {
  return 'CardStatus.expired(validUntil: $validUntil)';
}


}

/// @nodoc
abstract mixin class $CardStatusExpiredCopyWith<$Res> implements $CardStatusCopyWith<$Res> {
  factory $CardStatusExpiredCopyWith(CardStatusExpired value, $Res Function(CardStatusExpired) _then) = _$CardStatusExpiredCopyWithImpl;
@useResult
$Res call({
 DateTime validUntil
});




}
/// @nodoc
class _$CardStatusExpiredCopyWithImpl<$Res>
    implements $CardStatusExpiredCopyWith<$Res> {
  _$CardStatusExpiredCopyWithImpl(this._self, this._then);

  final CardStatusExpired _self;
  final $Res Function(CardStatusExpired) _then;

/// Create a copy of CardStatus
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? validUntil = null,}) {
  return _then(CardStatusExpired(
validUntil: null == validUntil ? _self.validUntil : validUntil // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}


}

/// @nodoc
@JsonSerializable()

class CardStatusRevoked implements CardStatus {
  const CardStatusRevoked({final  String? $type}): $type = $type ?? 'revoked';
  factory CardStatusRevoked.fromJson(Map<String, dynamic> json) => _$CardStatusRevokedFromJson(json);



@JsonKey(name: 'runtimeType')
final String $type;



@override
Map<String, dynamic> toJson() {
  return _$CardStatusRevokedToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardStatusRevoked);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CardStatus.revoked()';
}


}




/// @nodoc
@JsonSerializable()

class CardStatusCorrupted implements CardStatus {
  const CardStatusCorrupted({final  String? $type}): $type = $type ?? 'corrupted';
  factory CardStatusCorrupted.fromJson(Map<String, dynamic> json) => _$CardStatusCorruptedFromJson(json);



@JsonKey(name: 'runtimeType')
final String $type;



@override
Map<String, dynamic> toJson() {
  return _$CardStatusCorruptedToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardStatusCorrupted);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CardStatus.corrupted()';
}


}




/// @nodoc
@JsonSerializable()

class CardStatusUndetermined implements CardStatus {
  const CardStatusUndetermined({final  String? $type}): $type = $type ?? 'undetermined';
  factory CardStatusUndetermined.fromJson(Map<String, dynamic> json) => _$CardStatusUndeterminedFromJson(json);



@JsonKey(name: 'runtimeType')
final String $type;



@override
Map<String, dynamic> toJson() {
  return _$CardStatusUndeterminedToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardStatusUndetermined);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CardStatus.undetermined()';
}


}




// dart format on
