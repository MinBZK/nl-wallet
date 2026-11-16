// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'disclose_for_issuance_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DiscloseForIssuanceResult {

 List<WalletCard> get cards;
/// Create a copy of DiscloseForIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DiscloseForIssuanceResultCopyWith<DiscloseForIssuanceResult> get copyWith => _$DiscloseForIssuanceResultCopyWithImpl<DiscloseForIssuanceResult>(this as DiscloseForIssuanceResult, _$identity);



@override
bool operator ==(Object other) {
  final _this = this as DiscloseForIssuanceResult;
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DiscloseForIssuanceResult&&const DeepCollectionEquality().equals(other.cards, _this.cards));
}


@override
int get hashCode {
  final _this = this as DiscloseForIssuanceResult;
  return Object.hash(runtimeType,const DeepCollectionEquality().hash(_this.cards));
}

@override
String toString() {
  final _this = this as DiscloseForIssuanceResult;
  return 'DiscloseForIssuanceResult(cards: ${_this.cards})';
}


}

/// @nodoc
abstract mixin class $DiscloseForIssuanceResultCopyWith<$Res>  {
  factory $DiscloseForIssuanceResultCopyWith(DiscloseForIssuanceResult value, $Res Function(DiscloseForIssuanceResult) _then) = _$DiscloseForIssuanceResultCopyWithImpl;
@useResult
$Res call({
 List<WalletCard> cards
});




}
/// @nodoc
class _$DiscloseForIssuanceResultCopyWithImpl<$Res>
    implements $DiscloseForIssuanceResultCopyWith<$Res> {
  _$DiscloseForIssuanceResultCopyWithImpl(this._self, this._then);

  final DiscloseForIssuanceResult _self;
  final $Res Function(DiscloseForIssuanceResult) _then;

/// Create a copy of DiscloseForIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? cards = null,}) {
  return _then(_self.copyWith(
cards: null == cards ? _self.cards : cards // ignore: cast_nullable_to_non_nullable
as List<WalletCard>,
  ));
}

}


/// Adds pattern-matching-related methods to [DiscloseForIssuanceResult].
extension DiscloseForIssuanceResultPatterns on DiscloseForIssuanceResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DiscloseForIssuancePreviews value)?  previews,TResult Function( DiscloseForIssuanceIssuedForReview value)?  issuedForReview,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DiscloseForIssuancePreviews() when previews != null:
return previews(_that);case DiscloseForIssuanceIssuedForReview() when issuedForReview != null:
return issuedForReview(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DiscloseForIssuancePreviews value)  previews,required TResult Function( DiscloseForIssuanceIssuedForReview value)  issuedForReview,}){
final _that = this;
switch (_that) {
case DiscloseForIssuancePreviews():
return previews(_that);case DiscloseForIssuanceIssuedForReview():
return issuedForReview(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DiscloseForIssuancePreviews value)?  previews,TResult? Function( DiscloseForIssuanceIssuedForReview value)?  issuedForReview,}){
final _that = this;
switch (_that) {
case DiscloseForIssuancePreviews() when previews != null:
return previews(_that);case DiscloseForIssuanceIssuedForReview() when issuedForReview != null:
return issuedForReview(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( List<WalletCard> cards)?  previews,TResult Function( List<WalletCard> cards)?  issuedForReview,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DiscloseForIssuancePreviews() when previews != null:
return previews(_that.cards);case DiscloseForIssuanceIssuedForReview() when issuedForReview != null:
return issuedForReview(_that.cards);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( List<WalletCard> cards)  previews,required TResult Function( List<WalletCard> cards)  issuedForReview,}) {final _that = this;
switch (_that) {
case DiscloseForIssuancePreviews():
return previews(_that.cards);case DiscloseForIssuanceIssuedForReview():
return issuedForReview(_that.cards);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( List<WalletCard> cards)?  previews,TResult? Function( List<WalletCard> cards)?  issuedForReview,}) {final _that = this;
switch (_that) {
case DiscloseForIssuancePreviews() when previews != null:
return previews(_that.cards);case DiscloseForIssuanceIssuedForReview() when issuedForReview != null:
return issuedForReview(_that.cards);case _:
  return null;

}
}

}

/// @nodoc


class DiscloseForIssuancePreviews implements DiscloseForIssuanceResult {
  const DiscloseForIssuancePreviews( List<WalletCard> cards): _cards = cards;
  

 final  List<WalletCard> _cards;
@override List<WalletCard> get cards {
  if (_cards is EqualUnmodifiableListView) return _cards;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_cards);
}


/// Create a copy of DiscloseForIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DiscloseForIssuancePreviewsCopyWith<DiscloseForIssuancePreviews> get copyWith => _$DiscloseForIssuancePreviewsCopyWithImpl<DiscloseForIssuancePreviews>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is DiscloseForIssuancePreviews&&const DeepCollectionEquality().equals(other.cards, _cards));
}


@override
int get hashCode {
    return Object.hash(runtimeType,const DeepCollectionEquality().hash(_cards));
}

@override
String toString() {
    return 'DiscloseForIssuanceResult.previews(cards: $cards)';
}


}

/// @nodoc
abstract mixin class $DiscloseForIssuancePreviewsCopyWith<$Res> implements $DiscloseForIssuanceResultCopyWith<$Res> {
  factory $DiscloseForIssuancePreviewsCopyWith(DiscloseForIssuancePreviews value, $Res Function(DiscloseForIssuancePreviews) _then) = _$DiscloseForIssuancePreviewsCopyWithImpl;
@override @useResult
$Res call({
 List<WalletCard> cards
});




}
/// @nodoc
class _$DiscloseForIssuancePreviewsCopyWithImpl<$Res>
    implements $DiscloseForIssuancePreviewsCopyWith<$Res> {
  _$DiscloseForIssuancePreviewsCopyWithImpl(this._self, this._then);

  final DiscloseForIssuancePreviews _self;
  final $Res Function(DiscloseForIssuancePreviews) _then;

/// Create a copy of DiscloseForIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? cards = null,}) {
  return _then(DiscloseForIssuancePreviews(
null == cards ? _self._cards : cards // ignore: cast_nullable_to_non_nullable
as List<WalletCard>,
  ));
}


}

/// @nodoc


class DiscloseForIssuanceIssuedForReview implements DiscloseForIssuanceResult {
  const DiscloseForIssuanceIssuedForReview( List<WalletCard> cards): _cards = cards;
  

 final  List<WalletCard> _cards;
@override List<WalletCard> get cards {
  if (_cards is EqualUnmodifiableListView) return _cards;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_cards);
}


/// Create a copy of DiscloseForIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DiscloseForIssuanceIssuedForReviewCopyWith<DiscloseForIssuanceIssuedForReview> get copyWith => _$DiscloseForIssuanceIssuedForReviewCopyWithImpl<DiscloseForIssuanceIssuedForReview>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is DiscloseForIssuanceIssuedForReview&&const DeepCollectionEquality().equals(other.cards, _cards));
}


@override
int get hashCode {
    return Object.hash(runtimeType,const DeepCollectionEquality().hash(_cards));
}

@override
String toString() {
    return 'DiscloseForIssuanceResult.issuedForReview(cards: $cards)';
}


}

/// @nodoc
abstract mixin class $DiscloseForIssuanceIssuedForReviewCopyWith<$Res> implements $DiscloseForIssuanceResultCopyWith<$Res> {
  factory $DiscloseForIssuanceIssuedForReviewCopyWith(DiscloseForIssuanceIssuedForReview value, $Res Function(DiscloseForIssuanceIssuedForReview) _then) = _$DiscloseForIssuanceIssuedForReviewCopyWithImpl;
@override @useResult
$Res call({
 List<WalletCard> cards
});




}
/// @nodoc
class _$DiscloseForIssuanceIssuedForReviewCopyWithImpl<$Res>
    implements $DiscloseForIssuanceIssuedForReviewCopyWith<$Res> {
  _$DiscloseForIssuanceIssuedForReviewCopyWithImpl(this._self, this._then);

  final DiscloseForIssuanceIssuedForReview _self;
  final $Res Function(DiscloseForIssuanceIssuedForReview) _then;

/// Create a copy of DiscloseForIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? cards = null,}) {
  return _then(DiscloseForIssuanceIssuedForReview(
null == cards ? _self._cards : cards // ignore: cast_nullable_to_non_nullable
as List<WalletCard>,
  ));
}


}

// dart format on
