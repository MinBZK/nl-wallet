// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'card_display_metadata.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$CardDisplayMetadata {

@LocaleConverter() Locale get language; String get name; String? get description; String? get rawSummary;@CardRenderingConverter() CardRendering? get rendering;

  /// Serializes this CardDisplayMetadata to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CardDisplayMetadata&&(identical(other.language, language) || other.language == language)&&(identical(other.name, name) || other.name == name)&&(identical(other.description, description) || other.description == description)&&(identical(other.rawSummary, rawSummary) || other.rawSummary == rawSummary)&&(identical(other.rendering, rendering) || other.rendering == rendering));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,language,name,description,rawSummary,rendering);

@override
String toString() {
  return 'CardDisplayMetadata(language: $language, name: $name, description: $description, rawSummary: $rawSummary, rendering: $rendering)';
}


}




/// Adds pattern-matching-related methods to [CardDisplayMetadata].
extension CardDisplayMetadataPatterns on CardDisplayMetadata {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CardDisplayMetadata value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CardDisplayMetadata() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CardDisplayMetadata value)  $default,){
final _that = this;
switch (_that) {
case _CardDisplayMetadata():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CardDisplayMetadata value)?  $default,){
final _that = this;
switch (_that) {
case _CardDisplayMetadata() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@LocaleConverter()  Locale language,  String name,  String? description,  String? rawSummary, @CardRenderingConverter()  CardRendering? rendering)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CardDisplayMetadata() when $default != null:
return $default(_that.language,_that.name,_that.description,_that.rawSummary,_that.rendering);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@LocaleConverter()  Locale language,  String name,  String? description,  String? rawSummary, @CardRenderingConverter()  CardRendering? rendering)  $default,) {final _that = this;
switch (_that) {
case _CardDisplayMetadata():
return $default(_that.language,_that.name,_that.description,_that.rawSummary,_that.rendering);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@LocaleConverter()  Locale language,  String name,  String? description,  String? rawSummary, @CardRenderingConverter()  CardRendering? rendering)?  $default,) {final _that = this;
switch (_that) {
case _CardDisplayMetadata() when $default != null:
return $default(_that.language,_that.name,_that.description,_that.rawSummary,_that.rendering);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _CardDisplayMetadata implements CardDisplayMetadata {
  const _CardDisplayMetadata({@LocaleConverter() required this.language, required this.name, this.description, this.rawSummary, @CardRenderingConverter() this.rendering});
  factory _CardDisplayMetadata.fromJson(Map<String, dynamic> json) => _$CardDisplayMetadataFromJson(json);

@override@LocaleConverter() final  Locale language;
@override final  String name;
@override final  String? description;
@override final  String? rawSummary;
@override@CardRenderingConverter() final  CardRendering? rendering;


@override
Map<String, dynamic> toJson() {
  return _$CardDisplayMetadataToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CardDisplayMetadata&&(identical(other.language, language) || other.language == language)&&(identical(other.name, name) || other.name == name)&&(identical(other.description, description) || other.description == description)&&(identical(other.rawSummary, rawSummary) || other.rawSummary == rawSummary)&&(identical(other.rendering, rendering) || other.rendering == rendering));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,language,name,description,rawSummary,rendering);

@override
String toString() {
  return 'CardDisplayMetadata(language: $language, name: $name, description: $description, rawSummary: $rawSummary, rendering: $rendering)';
}


}




// dart format on
