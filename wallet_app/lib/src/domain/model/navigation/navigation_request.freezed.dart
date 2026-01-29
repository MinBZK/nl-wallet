// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'navigation_request.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
NavigationRequest _$NavigationRequestFromJson(
  Map<String, dynamic> json
) {
    return GenericNavigationRequest.fromJson(
      json
    );
}

/// @nodoc
mixin _$NavigationRequest {

 String get destination; String? get removeUntil; Object? get argument; List<NavigationPrerequisite> get navigatePrerequisites; List<PreNavigationAction> get preNavigationActions;
/// Create a copy of NavigationRequest
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NavigationRequestCopyWith<NavigationRequest> get copyWith => _$NavigationRequestCopyWithImpl<NavigationRequest>(this as NavigationRequest, _$identity);

  /// Serializes this NavigationRequest to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationRequest&&(identical(other.destination, destination) || other.destination == destination)&&(identical(other.removeUntil, removeUntil) || other.removeUntil == removeUntil)&&const DeepCollectionEquality().equals(other.argument, argument)&&const DeepCollectionEquality().equals(other.navigatePrerequisites, navigatePrerequisites)&&const DeepCollectionEquality().equals(other.preNavigationActions, preNavigationActions));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,destination,removeUntil,const DeepCollectionEquality().hash(argument),const DeepCollectionEquality().hash(navigatePrerequisites),const DeepCollectionEquality().hash(preNavigationActions));

@override
String toString() {
  return 'NavigationRequest(destination: $destination, removeUntil: $removeUntil, argument: $argument, navigatePrerequisites: $navigatePrerequisites, preNavigationActions: $preNavigationActions)';
}


}

/// @nodoc
abstract mixin class $NavigationRequestCopyWith<$Res>  {
  factory $NavigationRequestCopyWith(NavigationRequest value, $Res Function(NavigationRequest) _then) = _$NavigationRequestCopyWithImpl;
@useResult
$Res call({
 String destination, String? removeUntil, Object? argument, List<NavigationPrerequisite> navigatePrerequisites, List<PreNavigationAction> preNavigationActions
});




}
/// @nodoc
class _$NavigationRequestCopyWithImpl<$Res>
    implements $NavigationRequestCopyWith<$Res> {
  _$NavigationRequestCopyWithImpl(this._self, this._then);

  final NavigationRequest _self;
  final $Res Function(NavigationRequest) _then;

/// Create a copy of NavigationRequest
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? destination = null,Object? removeUntil = freezed,Object? argument = freezed,Object? navigatePrerequisites = null,Object? preNavigationActions = null,}) {
  return _then(_self.copyWith(
destination: null == destination ? _self.destination : destination // ignore: cast_nullable_to_non_nullable
as String,removeUntil: freezed == removeUntil ? _self.removeUntil : removeUntil // ignore: cast_nullable_to_non_nullable
as String?,argument: freezed == argument ? _self.argument : argument ,navigatePrerequisites: null == navigatePrerequisites ? _self.navigatePrerequisites : navigatePrerequisites // ignore: cast_nullable_to_non_nullable
as List<NavigationPrerequisite>,preNavigationActions: null == preNavigationActions ? _self.preNavigationActions : preNavigationActions // ignore: cast_nullable_to_non_nullable
as List<PreNavigationAction>,
  ));
}

}


/// Adds pattern-matching-related methods to [NavigationRequest].
extension NavigationRequestPatterns on NavigationRequest {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( GenericNavigationRequest value)?  generic,required TResult orElse(),}){
final _that = this;
switch (_that) {
case GenericNavigationRequest() when generic != null:
return generic(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( GenericNavigationRequest value)  generic,}){
final _that = this;
switch (_that) {
case GenericNavigationRequest():
return generic(_that);case _:
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( GenericNavigationRequest value)?  generic,}){
final _that = this;
switch (_that) {
case GenericNavigationRequest() when generic != null:
return generic(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String destination,  String? removeUntil,  Object? argument,  List<NavigationPrerequisite> navigatePrerequisites,  List<PreNavigationAction> preNavigationActions)?  generic,required TResult orElse(),}) {final _that = this;
switch (_that) {
case GenericNavigationRequest() when generic != null:
return generic(_that.destination,_that.removeUntil,_that.argument,_that.navigatePrerequisites,_that.preNavigationActions);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String destination,  String? removeUntil,  Object? argument,  List<NavigationPrerequisite> navigatePrerequisites,  List<PreNavigationAction> preNavigationActions)  generic,}) {final _that = this;
switch (_that) {
case GenericNavigationRequest():
return generic(_that.destination,_that.removeUntil,_that.argument,_that.navigatePrerequisites,_that.preNavigationActions);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String destination,  String? removeUntil,  Object? argument,  List<NavigationPrerequisite> navigatePrerequisites,  List<PreNavigationAction> preNavigationActions)?  generic,}) {final _that = this;
switch (_that) {
case GenericNavigationRequest() when generic != null:
return generic(_that.destination,_that.removeUntil,_that.argument,_that.navigatePrerequisites,_that.preNavigationActions);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class GenericNavigationRequest extends NavigationRequest {
  const GenericNavigationRequest(this.destination, {this.removeUntil, this.argument, final  List<NavigationPrerequisite> navigatePrerequisites = const [], final  List<PreNavigationAction> preNavigationActions = const []}): _navigatePrerequisites = navigatePrerequisites,_preNavigationActions = preNavigationActions,super._();
  factory GenericNavigationRequest.fromJson(Map<String, dynamic> json) => _$GenericNavigationRequestFromJson(json);

@override final  String destination;
@override final  String? removeUntil;
@override final  Object? argument;
 final  List<NavigationPrerequisite> _navigatePrerequisites;
@override@JsonKey() List<NavigationPrerequisite> get navigatePrerequisites {
  if (_navigatePrerequisites is EqualUnmodifiableListView) return _navigatePrerequisites;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_navigatePrerequisites);
}

 final  List<PreNavigationAction> _preNavigationActions;
@override@JsonKey() List<PreNavigationAction> get preNavigationActions {
  if (_preNavigationActions is EqualUnmodifiableListView) return _preNavigationActions;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_preNavigationActions);
}


/// Create a copy of NavigationRequest
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GenericNavigationRequestCopyWith<GenericNavigationRequest> get copyWith => _$GenericNavigationRequestCopyWithImpl<GenericNavigationRequest>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$GenericNavigationRequestToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GenericNavigationRequest&&(identical(other.destination, destination) || other.destination == destination)&&(identical(other.removeUntil, removeUntil) || other.removeUntil == removeUntil)&&const DeepCollectionEquality().equals(other.argument, argument)&&const DeepCollectionEquality().equals(other._navigatePrerequisites, _navigatePrerequisites)&&const DeepCollectionEquality().equals(other._preNavigationActions, _preNavigationActions));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,destination,removeUntil,const DeepCollectionEquality().hash(argument),const DeepCollectionEquality().hash(_navigatePrerequisites),const DeepCollectionEquality().hash(_preNavigationActions));

@override
String toString() {
  return 'NavigationRequest.generic(destination: $destination, removeUntil: $removeUntil, argument: $argument, navigatePrerequisites: $navigatePrerequisites, preNavigationActions: $preNavigationActions)';
}


}

/// @nodoc
abstract mixin class $GenericNavigationRequestCopyWith<$Res> implements $NavigationRequestCopyWith<$Res> {
  factory $GenericNavigationRequestCopyWith(GenericNavigationRequest value, $Res Function(GenericNavigationRequest) _then) = _$GenericNavigationRequestCopyWithImpl;
@override @useResult
$Res call({
 String destination, String? removeUntil, Object? argument, List<NavigationPrerequisite> navigatePrerequisites, List<PreNavigationAction> preNavigationActions
});




}
/// @nodoc
class _$GenericNavigationRequestCopyWithImpl<$Res>
    implements $GenericNavigationRequestCopyWith<$Res> {
  _$GenericNavigationRequestCopyWithImpl(this._self, this._then);

  final GenericNavigationRequest _self;
  final $Res Function(GenericNavigationRequest) _then;

/// Create a copy of NavigationRequest
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? destination = null,Object? removeUntil = freezed,Object? argument = freezed,Object? navigatePrerequisites = null,Object? preNavigationActions = null,}) {
  return _then(GenericNavigationRequest(
null == destination ? _self.destination : destination // ignore: cast_nullable_to_non_nullable
as String,removeUntil: freezed == removeUntil ? _self.removeUntil : removeUntil // ignore: cast_nullable_to_non_nullable
as String?,argument: freezed == argument ? _self.argument : argument ,navigatePrerequisites: null == navigatePrerequisites ? _self._navigatePrerequisites : navigatePrerequisites // ignore: cast_nullable_to_non_nullable
as List<NavigationPrerequisite>,preNavigationActions: null == preNavigationActions ? _self._preNavigationActions : preNavigationActions // ignore: cast_nullable_to_non_nullable
as List<PreNavigationAction>,
  ));
}


}

// dart format on
