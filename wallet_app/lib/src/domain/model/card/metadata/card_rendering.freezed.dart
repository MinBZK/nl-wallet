// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'card_rendering.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
CardRendering _$CardRenderingFromJson(
  Map<String, dynamic> json
) {
    return SimpleCardRendering.fromJson(
      json
    );
}

/// @nodoc
mixin _$CardRendering {

@AppImageDataConverter() AppImageData? get logo; String? get logoAltText;@ColorConverter() Color? get bgColor;@ColorConverter() Color? get textColor;
/// Create a copy of CardRendering
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CardRenderingCopyWith<CardRendering> get copyWith => _$CardRenderingCopyWithImpl<CardRendering>(this as CardRendering, _$identity);

  /// Serializes this CardRendering to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardRendering&&(identical(other.logo, logo) || other.logo == logo)&&(identical(other.logoAltText, logoAltText) || other.logoAltText == logoAltText)&&(identical(other.bgColor, bgColor) || other.bgColor == bgColor)&&(identical(other.textColor, textColor) || other.textColor == textColor));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,logo,logoAltText,bgColor,textColor);

@override
String toString() {
  return 'CardRendering(logo: $logo, logoAltText: $logoAltText, bgColor: $bgColor, textColor: $textColor)';
}


}

/// @nodoc
abstract mixin class $CardRenderingCopyWith<$Res>  {
  factory $CardRenderingCopyWith(CardRendering value, $Res Function(CardRendering) _then) = _$CardRenderingCopyWithImpl;
@useResult
$Res call({
@AppImageDataConverter() AppImageData? logo, String? logoAltText,@ColorConverter() Color? bgColor,@ColorConverter() Color? textColor
});




}
/// @nodoc
class _$CardRenderingCopyWithImpl<$Res>
    implements $CardRenderingCopyWith<$Res> {
  _$CardRenderingCopyWithImpl(this._self, this._then);

  final CardRendering _self;
  final $Res Function(CardRendering) _then;

/// Create a copy of CardRendering
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? logo = freezed,Object? logoAltText = freezed,Object? bgColor = freezed,Object? textColor = freezed,}) {
  return _then(_self.copyWith(
logo: freezed == logo ? _self.logo : logo // ignore: cast_nullable_to_non_nullable
as AppImageData?,logoAltText: freezed == logoAltText ? _self.logoAltText : logoAltText // ignore: cast_nullable_to_non_nullable
as String?,bgColor: freezed == bgColor ? _self.bgColor : bgColor // ignore: cast_nullable_to_non_nullable
as Color?,textColor: freezed == textColor ? _self.textColor : textColor // ignore: cast_nullable_to_non_nullable
as Color?,
  ));
}

}


/// Adds pattern-matching-related methods to [CardRendering].
extension CardRenderingPatterns on CardRendering {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SimpleCardRendering value)?  simple,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SimpleCardRendering() when simple != null:
return simple(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SimpleCardRendering value)  simple,}){
final _that = this;
switch (_that) {
case SimpleCardRendering():
return simple(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SimpleCardRendering value)?  simple,}){
final _that = this;
switch (_that) {
case SimpleCardRendering() when simple != null:
return simple(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function(@AppImageDataConverter()  AppImageData? logo,  String? logoAltText, @ColorConverter()  Color? bgColor, @ColorConverter()  Color? textColor)?  simple,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SimpleCardRendering() when simple != null:
return simple(_that.logo,_that.logoAltText,_that.bgColor,_that.textColor);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function(@AppImageDataConverter()  AppImageData? logo,  String? logoAltText, @ColorConverter()  Color? bgColor, @ColorConverter()  Color? textColor)  simple,}) {final _that = this;
switch (_that) {
case SimpleCardRendering():
return simple(_that.logo,_that.logoAltText,_that.bgColor,_that.textColor);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function(@AppImageDataConverter()  AppImageData? logo,  String? logoAltText, @ColorConverter()  Color? bgColor, @ColorConverter()  Color? textColor)?  simple,}) {final _that = this;
switch (_that) {
case SimpleCardRendering() when simple != null:
return simple(_that.logo,_that.logoAltText,_that.bgColor,_that.textColor);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class SimpleCardRendering implements CardRendering {
  const SimpleCardRendering({@AppImageDataConverter() this.logo, this.logoAltText, @ColorConverter() this.bgColor, @ColorConverter() this.textColor});
  factory SimpleCardRendering.fromJson(Map<String, dynamic> json) => _$SimpleCardRenderingFromJson(json);

@override@AppImageDataConverter() final  AppImageData? logo;
@override final  String? logoAltText;
@override@ColorConverter() final  Color? bgColor;
@override@ColorConverter() final  Color? textColor;

/// Create a copy of CardRendering
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SimpleCardRenderingCopyWith<SimpleCardRendering> get copyWith => _$SimpleCardRenderingCopyWithImpl<SimpleCardRendering>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$SimpleCardRenderingToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SimpleCardRendering&&(identical(other.logo, logo) || other.logo == logo)&&(identical(other.logoAltText, logoAltText) || other.logoAltText == logoAltText)&&(identical(other.bgColor, bgColor) || other.bgColor == bgColor)&&(identical(other.textColor, textColor) || other.textColor == textColor));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,logo,logoAltText,bgColor,textColor);

@override
String toString() {
  return 'CardRendering.simple(logo: $logo, logoAltText: $logoAltText, bgColor: $bgColor, textColor: $textColor)';
}


}

/// @nodoc
abstract mixin class $SimpleCardRenderingCopyWith<$Res> implements $CardRenderingCopyWith<$Res> {
  factory $SimpleCardRenderingCopyWith(SimpleCardRendering value, $Res Function(SimpleCardRendering) _then) = _$SimpleCardRenderingCopyWithImpl;
@override @useResult
$Res call({
@AppImageDataConverter() AppImageData? logo, String? logoAltText,@ColorConverter() Color? bgColor,@ColorConverter() Color? textColor
});




}
/// @nodoc
class _$SimpleCardRenderingCopyWithImpl<$Res>
    implements $SimpleCardRenderingCopyWith<$Res> {
  _$SimpleCardRenderingCopyWithImpl(this._self, this._then);

  final SimpleCardRendering _self;
  final $Res Function(SimpleCardRendering) _then;

/// Create a copy of CardRendering
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? logo = freezed,Object? logoAltText = freezed,Object? bgColor = freezed,Object? textColor = freezed,}) {
  return _then(SimpleCardRendering(
logo: freezed == logo ? _self.logo : logo // ignore: cast_nullable_to_non_nullable
as AppImageData?,logoAltText: freezed == logoAltText ? _self.logoAltText : logoAltText // ignore: cast_nullable_to_non_nullable
as String?,bgColor: freezed == bgColor ? _self.bgColor : bgColor // ignore: cast_nullable_to_non_nullable
as Color?,textColor: freezed == textColor ? _self.textColor : textColor // ignore: cast_nullable_to_non_nullable
as Color?,
  ));
}


}

// dart format on
