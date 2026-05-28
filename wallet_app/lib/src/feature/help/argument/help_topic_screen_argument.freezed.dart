// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'help_topic_screen_argument.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$HelpTopicScreenArgument {

 String get topicId; List<String> get visitedTopicIds;

  /// Serializes this HelpTopicScreenArgument to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is HelpTopicScreenArgument&&(identical(other.topicId, topicId) || other.topicId == topicId)&&const DeepCollectionEquality().equals(other.visitedTopicIds, visitedTopicIds));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,topicId,const DeepCollectionEquality().hash(visitedTopicIds));

@override
String toString() {
  return 'HelpTopicScreenArgument(topicId: $topicId, visitedTopicIds: $visitedTopicIds)';
}


}




/// Adds pattern-matching-related methods to [HelpTopicScreenArgument].
extension HelpTopicScreenArgumentPatterns on HelpTopicScreenArgument {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _HelpTopicScreenArgument value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _HelpTopicScreenArgument() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _HelpTopicScreenArgument value)  $default,){
final _that = this;
switch (_that) {
case _HelpTopicScreenArgument():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _HelpTopicScreenArgument value)?  $default,){
final _that = this;
switch (_that) {
case _HelpTopicScreenArgument() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String topicId,  List<String> visitedTopicIds)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _HelpTopicScreenArgument() when $default != null:
return $default(_that.topicId,_that.visitedTopicIds);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String topicId,  List<String> visitedTopicIds)  $default,) {final _that = this;
switch (_that) {
case _HelpTopicScreenArgument():
return $default(_that.topicId,_that.visitedTopicIds);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String topicId,  List<String> visitedTopicIds)?  $default,) {final _that = this;
switch (_that) {
case _HelpTopicScreenArgument() when $default != null:
return $default(_that.topicId,_that.visitedTopicIds);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _HelpTopicScreenArgument implements HelpTopicScreenArgument {
  const _HelpTopicScreenArgument({required this.topicId, final  List<String> visitedTopicIds = const <String>[]}): _visitedTopicIds = visitedTopicIds;
  factory _HelpTopicScreenArgument.fromJson(Map<String, dynamic> json) => _$HelpTopicScreenArgumentFromJson(json);

@override final  String topicId;
 final  List<String> _visitedTopicIds;
@override@JsonKey() List<String> get visitedTopicIds {
  if (_visitedTopicIds is EqualUnmodifiableListView) return _visitedTopicIds;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_visitedTopicIds);
}



@override
Map<String, dynamic> toJson() {
  return _$HelpTopicScreenArgumentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _HelpTopicScreenArgument&&(identical(other.topicId, topicId) || other.topicId == topicId)&&const DeepCollectionEquality().equals(other._visitedTopicIds, _visitedTopicIds));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,topicId,const DeepCollectionEquality().hash(_visitedTopicIds));

@override
String toString() {
  return 'HelpTopicScreenArgument(topicId: $topicId, visitedTopicIds: $visitedTopicIds)';
}


}




// dart format on
