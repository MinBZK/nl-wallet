// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'card_detail_screen_argument.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$CardDetailScreenArgument {

 WalletCard? get card; String get cardId;@LocalizedTextConverter() LocalizedText get cardTitle;

  /// Serializes this CardDetailScreenArgument to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardDetailScreenArgument&&(identical(other.card, card) || other.card == card)&&(identical(other.cardId, cardId) || other.cardId == cardId)&&const DeepCollectionEquality().equals(other.cardTitle, cardTitle));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,card,cardId,const DeepCollectionEquality().hash(cardTitle));

@override
String toString() {
  return 'CardDetailScreenArgument(card: $card, cardId: $cardId, cardTitle: $cardTitle)';
}


}




/// Adds pattern-matching-related methods to [CardDetailScreenArgument].
extension CardDetailScreenArgumentPatterns on CardDetailScreenArgument {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CardDetailScreenArgument value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CardDetailScreenArgument() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CardDetailScreenArgument value)  $default,){
final _that = this;
switch (_that) {
case _CardDetailScreenArgument():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CardDetailScreenArgument value)?  $default,){
final _that = this;
switch (_that) {
case _CardDetailScreenArgument() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( WalletCard? card,  String cardId, @LocalizedTextConverter()  LocalizedText cardTitle)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CardDetailScreenArgument() when $default != null:
return $default(_that.card,_that.cardId,_that.cardTitle);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( WalletCard? card,  String cardId, @LocalizedTextConverter()  LocalizedText cardTitle)  $default,) {final _that = this;
switch (_that) {
case _CardDetailScreenArgument():
return $default(_that.card,_that.cardId,_that.cardTitle);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( WalletCard? card,  String cardId, @LocalizedTextConverter()  LocalizedText cardTitle)?  $default,) {final _that = this;
switch (_that) {
case _CardDetailScreenArgument() when $default != null:
return $default(_that.card,_that.cardId,_that.cardTitle);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _CardDetailScreenArgument extends CardDetailScreenArgument {
  const _CardDetailScreenArgument({this.card, required this.cardId, @LocalizedTextConverter() required final  LocalizedText cardTitle}): _cardTitle = cardTitle,super._();
  factory _CardDetailScreenArgument.fromJson(Map<String, dynamic> json) => _$CardDetailScreenArgumentFromJson(json);

@override final  WalletCard? card;
@override final  String cardId;
 final  LocalizedText _cardTitle;
@override@LocalizedTextConverter() LocalizedText get cardTitle {
  if (_cardTitle is EqualUnmodifiableMapView) return _cardTitle;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_cardTitle);
}



@override
Map<String, dynamic> toJson() {
  return _$CardDetailScreenArgumentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CardDetailScreenArgument&&(identical(other.card, card) || other.card == card)&&(identical(other.cardId, cardId) || other.cardId == cardId)&&const DeepCollectionEquality().equals(other._cardTitle, _cardTitle));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,card,cardId,const DeepCollectionEquality().hash(_cardTitle));

@override
String toString() {
  return 'CardDetailScreenArgument(card: $card, cardId: $cardId, cardTitle: $cardTitle)';
}


}




// dart format on
