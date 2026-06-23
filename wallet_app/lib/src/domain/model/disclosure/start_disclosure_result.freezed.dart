// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'start_disclosure_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$StartDisclosureResult {

 Organization get relyingParty; String get originUrl; LocalizedText get requestPurpose; bool get sharedDataWithOrganizationBefore; DisclosureSessionType get sessionType;
/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StartDisclosureResultCopyWith<StartDisclosureResult> get copyWith => _$StartDisclosureResultCopyWithImpl<StartDisclosureResult>(this as StartDisclosureResult, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartDisclosureResult&&(identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty)&&(identical(other.originUrl, originUrl) || other.originUrl == originUrl)&&const DeepCollectionEquality().equals(other.requestPurpose, requestPurpose)&&(identical(other.sharedDataWithOrganizationBefore, sharedDataWithOrganizationBefore) || other.sharedDataWithOrganizationBefore == sharedDataWithOrganizationBefore)&&(identical(other.sessionType, sessionType) || other.sessionType == sessionType));
}


@override
int get hashCode => Object.hash(runtimeType,relyingParty,originUrl,const DeepCollectionEquality().hash(requestPurpose),sharedDataWithOrganizationBefore,sessionType);

@override
String toString() {
  return 'StartDisclosureResult(relyingParty: $relyingParty, originUrl: $originUrl, requestPurpose: $requestPurpose, sharedDataWithOrganizationBefore: $sharedDataWithOrganizationBefore, sessionType: $sessionType)';
}


}

/// @nodoc
abstract mixin class $StartDisclosureResultCopyWith<$Res>  {
  factory $StartDisclosureResultCopyWith(StartDisclosureResult value, $Res Function(StartDisclosureResult) _then) = _$StartDisclosureResultCopyWithImpl;
@useResult
$Res call({
 Organization relyingParty, String originUrl, Map<Locale, String> requestPurpose, bool sharedDataWithOrganizationBefore, DisclosureSessionType sessionType
});


$OrganizationCopyWith<$Res> get relyingParty;

}
/// @nodoc
class _$StartDisclosureResultCopyWithImpl<$Res>
    implements $StartDisclosureResultCopyWith<$Res> {
  _$StartDisclosureResultCopyWithImpl(this._self, this._then);

  final StartDisclosureResult _self;
  final $Res Function(StartDisclosureResult) _then;

/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? relyingParty = null,Object? originUrl = null,Object? requestPurpose = null,Object? sharedDataWithOrganizationBefore = null,Object? sessionType = null,}) {
  return _then(_self.copyWith(
relyingParty: null == relyingParty ? _self.relyingParty : relyingParty // ignore: cast_nullable_to_non_nullable
as Organization,originUrl: null == originUrl ? _self.originUrl : originUrl // ignore: cast_nullable_to_non_nullable
as String,requestPurpose: null == requestPurpose ? _self.requestPurpose : requestPurpose // ignore: cast_nullable_to_non_nullable
as Map<Locale, String>,sharedDataWithOrganizationBefore: null == sharedDataWithOrganizationBefore ? _self.sharedDataWithOrganizationBefore : sharedDataWithOrganizationBefore // ignore: cast_nullable_to_non_nullable
as bool,sessionType: null == sessionType ? _self.sessionType : sessionType // ignore: cast_nullable_to_non_nullable
as DisclosureSessionType,
  ));
}
/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OrganizationCopyWith<$Res> get relyingParty {
  
  return $OrganizationCopyWith<$Res>(_self.relyingParty, (value) {
    return _then(_self.copyWith(relyingParty: value));
  });
}
}


/// Adds pattern-matching-related methods to [StartDisclosureResult].
extension StartDisclosureResultPatterns on StartDisclosureResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( StartDisclosureReadyToDisclose value)?  readyToDisclose,TResult Function( StartDisclosureMissingAttributes value)?  missingAttributes,required TResult orElse(),}){
final _that = this;
switch (_that) {
case StartDisclosureReadyToDisclose() when readyToDisclose != null:
return readyToDisclose(_that);case StartDisclosureMissingAttributes() when missingAttributes != null:
return missingAttributes(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( StartDisclosureReadyToDisclose value)  readyToDisclose,required TResult Function( StartDisclosureMissingAttributes value)  missingAttributes,}){
final _that = this;
switch (_that) {
case StartDisclosureReadyToDisclose():
return readyToDisclose(_that);case StartDisclosureMissingAttributes():
return missingAttributes(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( StartDisclosureReadyToDisclose value)?  readyToDisclose,TResult? Function( StartDisclosureMissingAttributes value)?  missingAttributes,}){
final _that = this;
switch (_that) {
case StartDisclosureReadyToDisclose() when readyToDisclose != null:
return readyToDisclose(_that);case StartDisclosureMissingAttributes() when missingAttributes != null:
return missingAttributes(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  bool sharedDataWithOrganizationBefore,  DisclosureSessionType sessionType,  List<DiscloseCardRequest> cardRequests,  Policy policy,  DisclosureType type)?  readyToDisclose,TResult Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  bool sharedDataWithOrganizationBefore,  DisclosureSessionType sessionType,  List<MissingAttribute> missingAttributes)?  missingAttributes,required TResult orElse(),}) {final _that = this;
switch (_that) {
case StartDisclosureReadyToDisclose() when readyToDisclose != null:
return readyToDisclose(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sharedDataWithOrganizationBefore,_that.sessionType,_that.cardRequests,_that.policy,_that.type);case StartDisclosureMissingAttributes() when missingAttributes != null:
return missingAttributes(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sharedDataWithOrganizationBefore,_that.sessionType,_that.missingAttributes);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  bool sharedDataWithOrganizationBefore,  DisclosureSessionType sessionType,  List<DiscloseCardRequest> cardRequests,  Policy policy,  DisclosureType type)  readyToDisclose,required TResult Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  bool sharedDataWithOrganizationBefore,  DisclosureSessionType sessionType,  List<MissingAttribute> missingAttributes)  missingAttributes,}) {final _that = this;
switch (_that) {
case StartDisclosureReadyToDisclose():
return readyToDisclose(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sharedDataWithOrganizationBefore,_that.sessionType,_that.cardRequests,_that.policy,_that.type);case StartDisclosureMissingAttributes():
return missingAttributes(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sharedDataWithOrganizationBefore,_that.sessionType,_that.missingAttributes);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  bool sharedDataWithOrganizationBefore,  DisclosureSessionType sessionType,  List<DiscloseCardRequest> cardRequests,  Policy policy,  DisclosureType type)?  readyToDisclose,TResult? Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  bool sharedDataWithOrganizationBefore,  DisclosureSessionType sessionType,  List<MissingAttribute> missingAttributes)?  missingAttributes,}) {final _that = this;
switch (_that) {
case StartDisclosureReadyToDisclose() when readyToDisclose != null:
return readyToDisclose(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sharedDataWithOrganizationBefore,_that.sessionType,_that.cardRequests,_that.policy,_that.type);case StartDisclosureMissingAttributes() when missingAttributes != null:
return missingAttributes(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sharedDataWithOrganizationBefore,_that.sessionType,_that.missingAttributes);case _:
  return null;

}
}

}

/// @nodoc


class StartDisclosureReadyToDisclose implements StartDisclosureResult {
  const StartDisclosureReadyToDisclose({required this.relyingParty, required this.originUrl, required final  LocalizedText requestPurpose, required this.sharedDataWithOrganizationBefore, required this.sessionType, required final  List<DiscloseCardRequest> cardRequests, required this.policy, required this.type}): _requestPurpose = requestPurpose,_cardRequests = cardRequests;
  

@override final  Organization relyingParty;
@override final  String originUrl;
 final  LocalizedText _requestPurpose;
@override LocalizedText get requestPurpose {
  if (_requestPurpose is EqualUnmodifiableMapView) return _requestPurpose;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_requestPurpose);
}

@override final  bool sharedDataWithOrganizationBefore;
@override final  DisclosureSessionType sessionType;
 final  List<DiscloseCardRequest> _cardRequests;
 List<DiscloseCardRequest> get cardRequests {
  if (_cardRequests is EqualUnmodifiableListView) return _cardRequests;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_cardRequests);
}

 final  Policy policy;
 final  DisclosureType type;

/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StartDisclosureReadyToDiscloseCopyWith<StartDisclosureReadyToDisclose> get copyWith => _$StartDisclosureReadyToDiscloseCopyWithImpl<StartDisclosureReadyToDisclose>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartDisclosureReadyToDisclose&&(identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty)&&(identical(other.originUrl, originUrl) || other.originUrl == originUrl)&&const DeepCollectionEquality().equals(other._requestPurpose, _requestPurpose)&&(identical(other.sharedDataWithOrganizationBefore, sharedDataWithOrganizationBefore) || other.sharedDataWithOrganizationBefore == sharedDataWithOrganizationBefore)&&(identical(other.sessionType, sessionType) || other.sessionType == sessionType)&&const DeepCollectionEquality().equals(other._cardRequests, _cardRequests)&&(identical(other.policy, policy) || other.policy == policy)&&(identical(other.type, type) || other.type == type));
}


@override
int get hashCode => Object.hash(runtimeType,relyingParty,originUrl,const DeepCollectionEquality().hash(_requestPurpose),sharedDataWithOrganizationBefore,sessionType,const DeepCollectionEquality().hash(_cardRequests),policy,type);

@override
String toString() {
  return 'StartDisclosureResult.readyToDisclose(relyingParty: $relyingParty, originUrl: $originUrl, requestPurpose: $requestPurpose, sharedDataWithOrganizationBefore: $sharedDataWithOrganizationBefore, sessionType: $sessionType, cardRequests: $cardRequests, policy: $policy, type: $type)';
}


}

/// @nodoc
abstract mixin class $StartDisclosureReadyToDiscloseCopyWith<$Res> implements $StartDisclosureResultCopyWith<$Res> {
  factory $StartDisclosureReadyToDiscloseCopyWith(StartDisclosureReadyToDisclose value, $Res Function(StartDisclosureReadyToDisclose) _then) = _$StartDisclosureReadyToDiscloseCopyWithImpl;
@override @useResult
$Res call({
 Organization relyingParty, String originUrl, LocalizedText requestPurpose, bool sharedDataWithOrganizationBefore, DisclosureSessionType sessionType, List<DiscloseCardRequest> cardRequests, Policy policy, DisclosureType type
});


@override $OrganizationCopyWith<$Res> get relyingParty;

}
/// @nodoc
class _$StartDisclosureReadyToDiscloseCopyWithImpl<$Res>
    implements $StartDisclosureReadyToDiscloseCopyWith<$Res> {
  _$StartDisclosureReadyToDiscloseCopyWithImpl(this._self, this._then);

  final StartDisclosureReadyToDisclose _self;
  final $Res Function(StartDisclosureReadyToDisclose) _then;

/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? relyingParty = null,Object? originUrl = null,Object? requestPurpose = null,Object? sharedDataWithOrganizationBefore = null,Object? sessionType = null,Object? cardRequests = null,Object? policy = null,Object? type = null,}) {
  return _then(StartDisclosureReadyToDisclose(
relyingParty: null == relyingParty ? _self.relyingParty : relyingParty // ignore: cast_nullable_to_non_nullable
as Organization,originUrl: null == originUrl ? _self.originUrl : originUrl // ignore: cast_nullable_to_non_nullable
as String,requestPurpose: null == requestPurpose ? _self._requestPurpose : requestPurpose // ignore: cast_nullable_to_non_nullable
as LocalizedText,sharedDataWithOrganizationBefore: null == sharedDataWithOrganizationBefore ? _self.sharedDataWithOrganizationBefore : sharedDataWithOrganizationBefore // ignore: cast_nullable_to_non_nullable
as bool,sessionType: null == sessionType ? _self.sessionType : sessionType // ignore: cast_nullable_to_non_nullable
as DisclosureSessionType,cardRequests: null == cardRequests ? _self._cardRequests : cardRequests // ignore: cast_nullable_to_non_nullable
as List<DiscloseCardRequest>,policy: null == policy ? _self.policy : policy // ignore: cast_nullable_to_non_nullable
as Policy,type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as DisclosureType,
  ));
}

/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OrganizationCopyWith<$Res> get relyingParty {
  
  return $OrganizationCopyWith<$Res>(_self.relyingParty, (value) {
    return _then(_self.copyWith(relyingParty: value));
  });
}
}

/// @nodoc


class StartDisclosureMissingAttributes implements StartDisclosureResult {
  const StartDisclosureMissingAttributes({required this.relyingParty, required this.originUrl, required final  LocalizedText requestPurpose, required this.sharedDataWithOrganizationBefore, required this.sessionType, required final  List<MissingAttribute> missingAttributes}): _requestPurpose = requestPurpose,_missingAttributes = missingAttributes;
  

@override final  Organization relyingParty;
@override final  String originUrl;
 final  LocalizedText _requestPurpose;
@override LocalizedText get requestPurpose {
  if (_requestPurpose is EqualUnmodifiableMapView) return _requestPurpose;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_requestPurpose);
}

@override final  bool sharedDataWithOrganizationBefore;
@override final  DisclosureSessionType sessionType;
 final  List<MissingAttribute> _missingAttributes;
 List<MissingAttribute> get missingAttributes {
  if (_missingAttributes is EqualUnmodifiableListView) return _missingAttributes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_missingAttributes);
}


/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StartDisclosureMissingAttributesCopyWith<StartDisclosureMissingAttributes> get copyWith => _$StartDisclosureMissingAttributesCopyWithImpl<StartDisclosureMissingAttributes>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartDisclosureMissingAttributes&&(identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty)&&(identical(other.originUrl, originUrl) || other.originUrl == originUrl)&&const DeepCollectionEquality().equals(other._requestPurpose, _requestPurpose)&&(identical(other.sharedDataWithOrganizationBefore, sharedDataWithOrganizationBefore) || other.sharedDataWithOrganizationBefore == sharedDataWithOrganizationBefore)&&(identical(other.sessionType, sessionType) || other.sessionType == sessionType)&&const DeepCollectionEquality().equals(other._missingAttributes, _missingAttributes));
}


@override
int get hashCode => Object.hash(runtimeType,relyingParty,originUrl,const DeepCollectionEquality().hash(_requestPurpose),sharedDataWithOrganizationBefore,sessionType,const DeepCollectionEquality().hash(_missingAttributes));

@override
String toString() {
  return 'StartDisclosureResult.missingAttributes(relyingParty: $relyingParty, originUrl: $originUrl, requestPurpose: $requestPurpose, sharedDataWithOrganizationBefore: $sharedDataWithOrganizationBefore, sessionType: $sessionType, missingAttributes: $missingAttributes)';
}


}

/// @nodoc
abstract mixin class $StartDisclosureMissingAttributesCopyWith<$Res> implements $StartDisclosureResultCopyWith<$Res> {
  factory $StartDisclosureMissingAttributesCopyWith(StartDisclosureMissingAttributes value, $Res Function(StartDisclosureMissingAttributes) _then) = _$StartDisclosureMissingAttributesCopyWithImpl;
@override @useResult
$Res call({
 Organization relyingParty, String originUrl, LocalizedText requestPurpose, bool sharedDataWithOrganizationBefore, DisclosureSessionType sessionType, List<MissingAttribute> missingAttributes
});


@override $OrganizationCopyWith<$Res> get relyingParty;

}
/// @nodoc
class _$StartDisclosureMissingAttributesCopyWithImpl<$Res>
    implements $StartDisclosureMissingAttributesCopyWith<$Res> {
  _$StartDisclosureMissingAttributesCopyWithImpl(this._self, this._then);

  final StartDisclosureMissingAttributes _self;
  final $Res Function(StartDisclosureMissingAttributes) _then;

/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? relyingParty = null,Object? originUrl = null,Object? requestPurpose = null,Object? sharedDataWithOrganizationBefore = null,Object? sessionType = null,Object? missingAttributes = null,}) {
  return _then(StartDisclosureMissingAttributes(
relyingParty: null == relyingParty ? _self.relyingParty : relyingParty // ignore: cast_nullable_to_non_nullable
as Organization,originUrl: null == originUrl ? _self.originUrl : originUrl // ignore: cast_nullable_to_non_nullable
as String,requestPurpose: null == requestPurpose ? _self._requestPurpose : requestPurpose // ignore: cast_nullable_to_non_nullable
as LocalizedText,sharedDataWithOrganizationBefore: null == sharedDataWithOrganizationBefore ? _self.sharedDataWithOrganizationBefore : sharedDataWithOrganizationBefore // ignore: cast_nullable_to_non_nullable
as bool,sessionType: null == sessionType ? _self.sessionType : sessionType // ignore: cast_nullable_to_non_nullable
as DisclosureSessionType,missingAttributes: null == missingAttributes ? _self._missingAttributes : missingAttributes // ignore: cast_nullable_to_non_nullable
as List<MissingAttribute>,
  ));
}

/// Create a copy of StartDisclosureResult
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OrganizationCopyWith<$Res> get relyingParty {
  
  return $OrganizationCopyWith<$Res>(_self.relyingParty, (value) {
    return _then(_self.copyWith(relyingParty: value));
  });
}
}

// dart format on
