// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'wallet_card.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$WalletCard {

/// ID of the attestation, null when the card is not persisted in the database
 String? get attestationId;/// Type of document
 String get attestationType;/// Organization that issued this card
 Organization get issuer;/// Card status (e.g. valid, expired, revoked)
 CardStatus get status;/// Data attributes stored in the card
 List<DataAttribute> get attributes;/// Card display metadata for UI rendering
 List<CardDisplayMetadata> get metadata;
/// Create a copy of WalletCard
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WalletCardCopyWith<WalletCard> get copyWith => _$WalletCardCopyWithImpl<WalletCard>(this as WalletCard, _$identity);

  /// Serializes this WalletCard to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletCard&&(identical(other.attestationId, attestationId) || other.attestationId == attestationId)&&(identical(other.attestationType, attestationType) || other.attestationType == attestationType)&&(identical(other.issuer, issuer) || other.issuer == issuer)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.attributes, attributes)&&const DeepCollectionEquality().equals(other.metadata, metadata));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,attestationId,attestationType,issuer,status,const DeepCollectionEquality().hash(attributes),const DeepCollectionEquality().hash(metadata));

@override
String toString() {
  return 'WalletCard(attestationId: $attestationId, attestationType: $attestationType, issuer: $issuer, status: $status, attributes: $attributes, metadata: $metadata)';
}


}

/// @nodoc
abstract mixin class $WalletCardCopyWith<$Res>  {
  factory $WalletCardCopyWith(WalletCard value, $Res Function(WalletCard) _then) = _$WalletCardCopyWithImpl;
@useResult
$Res call({
 String? attestationId, String attestationType, Organization issuer, CardStatus status, List<DataAttribute> attributes, List<CardDisplayMetadata> metadata
});


$OrganizationCopyWith<$Res> get issuer;$CardStatusCopyWith<$Res> get status;

}
/// @nodoc
class _$WalletCardCopyWithImpl<$Res>
    implements $WalletCardCopyWith<$Res> {
  _$WalletCardCopyWithImpl(this._self, this._then);

  final WalletCard _self;
  final $Res Function(WalletCard) _then;

/// Create a copy of WalletCard
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? attestationId = freezed,Object? attestationType = null,Object? issuer = null,Object? status = null,Object? attributes = null,Object? metadata = null,}) {
  return _then(_self.copyWith(
attestationId: freezed == attestationId ? _self.attestationId : attestationId // ignore: cast_nullable_to_non_nullable
as String?,attestationType: null == attestationType ? _self.attestationType : attestationType // ignore: cast_nullable_to_non_nullable
as String,issuer: null == issuer ? _self.issuer : issuer // ignore: cast_nullable_to_non_nullable
as Organization,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as CardStatus,attributes: null == attributes ? _self.attributes : attributes // ignore: cast_nullable_to_non_nullable
as List<DataAttribute>,metadata: null == metadata ? _self.metadata : metadata // ignore: cast_nullable_to_non_nullable
as List<CardDisplayMetadata>,
  ));
}
/// Create a copy of WalletCard
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OrganizationCopyWith<$Res> get issuer {
  
  return $OrganizationCopyWith<$Res>(_self.issuer, (value) {
    return _then(_self.copyWith(issuer: value));
  });
}/// Create a copy of WalletCard
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CardStatusCopyWith<$Res> get status {
  
  return $CardStatusCopyWith<$Res>(_self.status, (value) {
    return _then(_self.copyWith(status: value));
  });
}
}


/// Adds pattern-matching-related methods to [WalletCard].
extension WalletCardPatterns on WalletCard {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _WalletCard value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _WalletCard() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _WalletCard value)  $default,){
final _that = this;
switch (_that) {
case _WalletCard():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _WalletCard value)?  $default,){
final _that = this;
switch (_that) {
case _WalletCard() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String? attestationId,  String attestationType,  Organization issuer,  CardStatus status,  List<DataAttribute> attributes,  List<CardDisplayMetadata> metadata)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _WalletCard() when $default != null:
return $default(_that.attestationId,_that.attestationType,_that.issuer,_that.status,_that.attributes,_that.metadata);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String? attestationId,  String attestationType,  Organization issuer,  CardStatus status,  List<DataAttribute> attributes,  List<CardDisplayMetadata> metadata)  $default,) {final _that = this;
switch (_that) {
case _WalletCard():
return $default(_that.attestationId,_that.attestationType,_that.issuer,_that.status,_that.attributes,_that.metadata);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String? attestationId,  String attestationType,  Organization issuer,  CardStatus status,  List<DataAttribute> attributes,  List<CardDisplayMetadata> metadata)?  $default,) {final _that = this;
switch (_that) {
case _WalletCard() when $default != null:
return $default(_that.attestationId,_that.attestationType,_that.issuer,_that.status,_that.attributes,_that.metadata);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _WalletCard extends WalletCard {
  const _WalletCard({this.attestationId, required this.attestationType, required this.issuer, required this.status, required final  List<DataAttribute> attributes, final  List<CardDisplayMetadata> metadata = const []}): _attributes = attributes,_metadata = metadata,super._();
  factory _WalletCard.fromJson(Map<String, dynamic> json) => _$WalletCardFromJson(json);

/// ID of the attestation, null when the card is not persisted in the database
@override final  String? attestationId;
/// Type of document
@override final  String attestationType;
/// Organization that issued this card
@override final  Organization issuer;
/// Card status (e.g. valid, expired, revoked)
@override final  CardStatus status;
/// Data attributes stored in the card
 final  List<DataAttribute> _attributes;
/// Data attributes stored in the card
@override List<DataAttribute> get attributes {
  if (_attributes is EqualUnmodifiableListView) return _attributes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_attributes);
}

/// Card display metadata for UI rendering
 final  List<CardDisplayMetadata> _metadata;
/// Card display metadata for UI rendering
@override@JsonKey() List<CardDisplayMetadata> get metadata {
  if (_metadata is EqualUnmodifiableListView) return _metadata;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_metadata);
}


/// Create a copy of WalletCard
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$WalletCardCopyWith<_WalletCard> get copyWith => __$WalletCardCopyWithImpl<_WalletCard>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$WalletCardToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _WalletCard&&(identical(other.attestationId, attestationId) || other.attestationId == attestationId)&&(identical(other.attestationType, attestationType) || other.attestationType == attestationType)&&(identical(other.issuer, issuer) || other.issuer == issuer)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._attributes, _attributes)&&const DeepCollectionEquality().equals(other._metadata, _metadata));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,attestationId,attestationType,issuer,status,const DeepCollectionEquality().hash(_attributes),const DeepCollectionEquality().hash(_metadata));

@override
String toString() {
  return 'WalletCard(attestationId: $attestationId, attestationType: $attestationType, issuer: $issuer, status: $status, attributes: $attributes, metadata: $metadata)';
}


}

/// @nodoc
abstract mixin class _$WalletCardCopyWith<$Res> implements $WalletCardCopyWith<$Res> {
  factory _$WalletCardCopyWith(_WalletCard value, $Res Function(_WalletCard) _then) = __$WalletCardCopyWithImpl;
@override @useResult
$Res call({
 String? attestationId, String attestationType, Organization issuer, CardStatus status, List<DataAttribute> attributes, List<CardDisplayMetadata> metadata
});


@override $OrganizationCopyWith<$Res> get issuer;@override $CardStatusCopyWith<$Res> get status;

}
/// @nodoc
class __$WalletCardCopyWithImpl<$Res>
    implements _$WalletCardCopyWith<$Res> {
  __$WalletCardCopyWithImpl(this._self, this._then);

  final _WalletCard _self;
  final $Res Function(_WalletCard) _then;

/// Create a copy of WalletCard
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? attestationId = freezed,Object? attestationType = null,Object? issuer = null,Object? status = null,Object? attributes = null,Object? metadata = null,}) {
  return _then(_WalletCard(
attestationId: freezed == attestationId ? _self.attestationId : attestationId // ignore: cast_nullable_to_non_nullable
as String?,attestationType: null == attestationType ? _self.attestationType : attestationType // ignore: cast_nullable_to_non_nullable
as String,issuer: null == issuer ? _self.issuer : issuer // ignore: cast_nullable_to_non_nullable
as Organization,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as CardStatus,attributes: null == attributes ? _self._attributes : attributes // ignore: cast_nullable_to_non_nullable
as List<DataAttribute>,metadata: null == metadata ? _self._metadata : metadata // ignore: cast_nullable_to_non_nullable
as List<CardDisplayMetadata>,
  ));
}

/// Create a copy of WalletCard
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OrganizationCopyWith<$Res> get issuer {
  
  return $OrganizationCopyWith<$Res>(_self.issuer, (value) {
    return _then(_self.copyWith(issuer: value));
  });
}/// Create a copy of WalletCard
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CardStatusCopyWith<$Res> get status {
  
  return $CardStatusCopyWith<$Res>(_self.status, (value) {
    return _then(_self.copyWith(status: value));
  });
}
}

// dart format on
