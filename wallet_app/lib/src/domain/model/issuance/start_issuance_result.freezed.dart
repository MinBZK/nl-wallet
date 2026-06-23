// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'start_issuance_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$StartIssuanceResult {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartIssuanceResult);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'StartIssuanceResult()';
}


}

/// @nodoc
class $StartIssuanceResultCopyWith<$Res>  {
$StartIssuanceResultCopyWith(StartIssuanceResult _, $Res Function(StartIssuanceResult) __);
}


/// Adds pattern-matching-related methods to [StartIssuanceResult].
extension StartIssuanceResultPatterns on StartIssuanceResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( StartIssuanceAuthorizationRequired value)?  authorizationRequired,TResult Function( StartIssuancePreAuthorizedOffer value)?  preAuthorizedOffer,TResult Function( StartIssuanceReadyToDisclose value)?  readyToDisclose,TResult Function( StartIssuanceMissingAttributes value)?  missingAttributes,required TResult orElse(),}){
final _that = this;
switch (_that) {
case StartIssuanceAuthorizationRequired() when authorizationRequired != null:
return authorizationRequired(_that);case StartIssuancePreAuthorizedOffer() when preAuthorizedOffer != null:
return preAuthorizedOffer(_that);case StartIssuanceReadyToDisclose() when readyToDisclose != null:
return readyToDisclose(_that);case StartIssuanceMissingAttributes() when missingAttributes != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( StartIssuanceAuthorizationRequired value)  authorizationRequired,required TResult Function( StartIssuancePreAuthorizedOffer value)  preAuthorizedOffer,required TResult Function( StartIssuanceReadyToDisclose value)  readyToDisclose,required TResult Function( StartIssuanceMissingAttributes value)  missingAttributes,}){
final _that = this;
switch (_that) {
case StartIssuanceAuthorizationRequired():
return authorizationRequired(_that);case StartIssuancePreAuthorizedOffer():
return preAuthorizedOffer(_that);case StartIssuanceReadyToDisclose():
return readyToDisclose(_that);case StartIssuanceMissingAttributes():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( StartIssuanceAuthorizationRequired value)?  authorizationRequired,TResult? Function( StartIssuancePreAuthorizedOffer value)?  preAuthorizedOffer,TResult? Function( StartIssuanceReadyToDisclose value)?  readyToDisclose,TResult? Function( StartIssuanceMissingAttributes value)?  missingAttributes,}){
final _that = this;
switch (_that) {
case StartIssuanceAuthorizationRequired() when authorizationRequired != null:
return authorizationRequired(_that);case StartIssuancePreAuthorizedOffer() when preAuthorizedOffer != null:
return preAuthorizedOffer(_that);case StartIssuanceReadyToDisclose() when readyToDisclose != null:
return readyToDisclose(_that);case StartIssuanceMissingAttributes() when missingAttributes != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String authUrl)?  authorizationRequired,TResult Function( List<WalletCard> previews)?  preAuthorizedOffer,TResult Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  DisclosureSessionType sessionType,  DisclosureType type,  List<DiscloseCardRequest> cardRequests,  Policy policy,  bool sharedDataWithOrganizationBefore)?  readyToDisclose,TResult Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  DisclosureSessionType sessionType,  List<MissingAttribute> missingAttributes,  bool sharedDataWithOrganizationBefore)?  missingAttributes,required TResult orElse(),}) {final _that = this;
switch (_that) {
case StartIssuanceAuthorizationRequired() when authorizationRequired != null:
return authorizationRequired(_that.authUrl);case StartIssuancePreAuthorizedOffer() when preAuthorizedOffer != null:
return preAuthorizedOffer(_that.previews);case StartIssuanceReadyToDisclose() when readyToDisclose != null:
return readyToDisclose(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sessionType,_that.type,_that.cardRequests,_that.policy,_that.sharedDataWithOrganizationBefore);case StartIssuanceMissingAttributes() when missingAttributes != null:
return missingAttributes(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sessionType,_that.missingAttributes,_that.sharedDataWithOrganizationBefore);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String authUrl)  authorizationRequired,required TResult Function( List<WalletCard> previews)  preAuthorizedOffer,required TResult Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  DisclosureSessionType sessionType,  DisclosureType type,  List<DiscloseCardRequest> cardRequests,  Policy policy,  bool sharedDataWithOrganizationBefore)  readyToDisclose,required TResult Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  DisclosureSessionType sessionType,  List<MissingAttribute> missingAttributes,  bool sharedDataWithOrganizationBefore)  missingAttributes,}) {final _that = this;
switch (_that) {
case StartIssuanceAuthorizationRequired():
return authorizationRequired(_that.authUrl);case StartIssuancePreAuthorizedOffer():
return preAuthorizedOffer(_that.previews);case StartIssuanceReadyToDisclose():
return readyToDisclose(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sessionType,_that.type,_that.cardRequests,_that.policy,_that.sharedDataWithOrganizationBefore);case StartIssuanceMissingAttributes():
return missingAttributes(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sessionType,_that.missingAttributes,_that.sharedDataWithOrganizationBefore);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String authUrl)?  authorizationRequired,TResult? Function( List<WalletCard> previews)?  preAuthorizedOffer,TResult? Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  DisclosureSessionType sessionType,  DisclosureType type,  List<DiscloseCardRequest> cardRequests,  Policy policy,  bool sharedDataWithOrganizationBefore)?  readyToDisclose,TResult? Function( Organization relyingParty,  String originUrl,  LocalizedText requestPurpose,  DisclosureSessionType sessionType,  List<MissingAttribute> missingAttributes,  bool sharedDataWithOrganizationBefore)?  missingAttributes,}) {final _that = this;
switch (_that) {
case StartIssuanceAuthorizationRequired() when authorizationRequired != null:
return authorizationRequired(_that.authUrl);case StartIssuancePreAuthorizedOffer() when preAuthorizedOffer != null:
return preAuthorizedOffer(_that.previews);case StartIssuanceReadyToDisclose() when readyToDisclose != null:
return readyToDisclose(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sessionType,_that.type,_that.cardRequests,_that.policy,_that.sharedDataWithOrganizationBefore);case StartIssuanceMissingAttributes() when missingAttributes != null:
return missingAttributes(_that.relyingParty,_that.originUrl,_that.requestPurpose,_that.sessionType,_that.missingAttributes,_that.sharedDataWithOrganizationBefore);case _:
  return null;

}
}

}

/// @nodoc


class StartIssuanceAuthorizationRequired extends StartIssuanceResult {
  const StartIssuanceAuthorizationRequired(this.authUrl): super._();
  

 final  String authUrl;

/// Create a copy of StartIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StartIssuanceAuthorizationRequiredCopyWith<StartIssuanceAuthorizationRequired> get copyWith => _$StartIssuanceAuthorizationRequiredCopyWithImpl<StartIssuanceAuthorizationRequired>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartIssuanceAuthorizationRequired&&(identical(other.authUrl, authUrl) || other.authUrl == authUrl));
}


@override
int get hashCode => Object.hash(runtimeType,authUrl);

@override
String toString() {
  return 'StartIssuanceResult.authorizationRequired(authUrl: $authUrl)';
}


}

/// @nodoc
abstract mixin class $StartIssuanceAuthorizationRequiredCopyWith<$Res> implements $StartIssuanceResultCopyWith<$Res> {
  factory $StartIssuanceAuthorizationRequiredCopyWith(StartIssuanceAuthorizationRequired value, $Res Function(StartIssuanceAuthorizationRequired) _then) = _$StartIssuanceAuthorizationRequiredCopyWithImpl;
@useResult
$Res call({
 String authUrl
});




}
/// @nodoc
class _$StartIssuanceAuthorizationRequiredCopyWithImpl<$Res>
    implements $StartIssuanceAuthorizationRequiredCopyWith<$Res> {
  _$StartIssuanceAuthorizationRequiredCopyWithImpl(this._self, this._then);

  final StartIssuanceAuthorizationRequired _self;
  final $Res Function(StartIssuanceAuthorizationRequired) _then;

/// Create a copy of StartIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? authUrl = null,}) {
  return _then(StartIssuanceAuthorizationRequired(
null == authUrl ? _self.authUrl : authUrl // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class StartIssuancePreAuthorizedOffer extends StartIssuanceResult {
  const StartIssuancePreAuthorizedOffer(final  List<WalletCard> previews): _previews = previews,super._();
  

 final  List<WalletCard> _previews;
 List<WalletCard> get previews {
  if (_previews is EqualUnmodifiableListView) return _previews;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_previews);
}


/// Create a copy of StartIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StartIssuancePreAuthorizedOfferCopyWith<StartIssuancePreAuthorizedOffer> get copyWith => _$StartIssuancePreAuthorizedOfferCopyWithImpl<StartIssuancePreAuthorizedOffer>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartIssuancePreAuthorizedOffer&&const DeepCollectionEquality().equals(other._previews, _previews));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_previews));

@override
String toString() {
  return 'StartIssuanceResult.preAuthorizedOffer(previews: $previews)';
}


}

/// @nodoc
abstract mixin class $StartIssuancePreAuthorizedOfferCopyWith<$Res> implements $StartIssuanceResultCopyWith<$Res> {
  factory $StartIssuancePreAuthorizedOfferCopyWith(StartIssuancePreAuthorizedOffer value, $Res Function(StartIssuancePreAuthorizedOffer) _then) = _$StartIssuancePreAuthorizedOfferCopyWithImpl;
@useResult
$Res call({
 List<WalletCard> previews
});




}
/// @nodoc
class _$StartIssuancePreAuthorizedOfferCopyWithImpl<$Res>
    implements $StartIssuancePreAuthorizedOfferCopyWith<$Res> {
  _$StartIssuancePreAuthorizedOfferCopyWithImpl(this._self, this._then);

  final StartIssuancePreAuthorizedOffer _self;
  final $Res Function(StartIssuancePreAuthorizedOffer) _then;

/// Create a copy of StartIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? previews = null,}) {
  return _then(StartIssuancePreAuthorizedOffer(
null == previews ? _self._previews : previews // ignore: cast_nullable_to_non_nullable
as List<WalletCard>,
  ));
}


}

/// @nodoc


class StartIssuanceReadyToDisclose extends StartIssuanceResult {
  const StartIssuanceReadyToDisclose({required this.relyingParty, required this.originUrl, required final  LocalizedText requestPurpose, required this.sessionType, required this.type, required final  List<DiscloseCardRequest> cardRequests, required this.policy, required this.sharedDataWithOrganizationBefore}): _requestPurpose = requestPurpose,_cardRequests = cardRequests,super._();
  

 final  Organization relyingParty;
 final  String originUrl;
 final  LocalizedText _requestPurpose;
 LocalizedText get requestPurpose {
  if (_requestPurpose is EqualUnmodifiableMapView) return _requestPurpose;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_requestPurpose);
}

 final  DisclosureSessionType sessionType;
 final  DisclosureType type;
 final  List<DiscloseCardRequest> _cardRequests;
 List<DiscloseCardRequest> get cardRequests {
  if (_cardRequests is EqualUnmodifiableListView) return _cardRequests;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_cardRequests);
}

 final  Policy policy;
 final  bool sharedDataWithOrganizationBefore;

/// Create a copy of StartIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StartIssuanceReadyToDiscloseCopyWith<StartIssuanceReadyToDisclose> get copyWith => _$StartIssuanceReadyToDiscloseCopyWithImpl<StartIssuanceReadyToDisclose>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartIssuanceReadyToDisclose&&(identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty)&&(identical(other.originUrl, originUrl) || other.originUrl == originUrl)&&const DeepCollectionEquality().equals(other._requestPurpose, _requestPurpose)&&(identical(other.sessionType, sessionType) || other.sessionType == sessionType)&&(identical(other.type, type) || other.type == type)&&const DeepCollectionEquality().equals(other._cardRequests, _cardRequests)&&(identical(other.policy, policy) || other.policy == policy)&&(identical(other.sharedDataWithOrganizationBefore, sharedDataWithOrganizationBefore) || other.sharedDataWithOrganizationBefore == sharedDataWithOrganizationBefore));
}


@override
int get hashCode => Object.hash(runtimeType,relyingParty,originUrl,const DeepCollectionEquality().hash(_requestPurpose),sessionType,type,const DeepCollectionEquality().hash(_cardRequests),policy,sharedDataWithOrganizationBefore);

@override
String toString() {
  return 'StartIssuanceResult.readyToDisclose(relyingParty: $relyingParty, originUrl: $originUrl, requestPurpose: $requestPurpose, sessionType: $sessionType, type: $type, cardRequests: $cardRequests, policy: $policy, sharedDataWithOrganizationBefore: $sharedDataWithOrganizationBefore)';
}


}

/// @nodoc
abstract mixin class $StartIssuanceReadyToDiscloseCopyWith<$Res> implements $StartIssuanceResultCopyWith<$Res> {
  factory $StartIssuanceReadyToDiscloseCopyWith(StartIssuanceReadyToDisclose value, $Res Function(StartIssuanceReadyToDisclose) _then) = _$StartIssuanceReadyToDiscloseCopyWithImpl;
@useResult
$Res call({
 Organization relyingParty, String originUrl, LocalizedText requestPurpose, DisclosureSessionType sessionType, DisclosureType type, List<DiscloseCardRequest> cardRequests, Policy policy, bool sharedDataWithOrganizationBefore
});


$OrganizationCopyWith<$Res> get relyingParty;

}
/// @nodoc
class _$StartIssuanceReadyToDiscloseCopyWithImpl<$Res>
    implements $StartIssuanceReadyToDiscloseCopyWith<$Res> {
  _$StartIssuanceReadyToDiscloseCopyWithImpl(this._self, this._then);

  final StartIssuanceReadyToDisclose _self;
  final $Res Function(StartIssuanceReadyToDisclose) _then;

/// Create a copy of StartIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? relyingParty = null,Object? originUrl = null,Object? requestPurpose = null,Object? sessionType = null,Object? type = null,Object? cardRequests = null,Object? policy = null,Object? sharedDataWithOrganizationBefore = null,}) {
  return _then(StartIssuanceReadyToDisclose(
relyingParty: null == relyingParty ? _self.relyingParty : relyingParty // ignore: cast_nullable_to_non_nullable
as Organization,originUrl: null == originUrl ? _self.originUrl : originUrl // ignore: cast_nullable_to_non_nullable
as String,requestPurpose: null == requestPurpose ? _self._requestPurpose : requestPurpose // ignore: cast_nullable_to_non_nullable
as LocalizedText,sessionType: null == sessionType ? _self.sessionType : sessionType // ignore: cast_nullable_to_non_nullable
as DisclosureSessionType,type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as DisclosureType,cardRequests: null == cardRequests ? _self._cardRequests : cardRequests // ignore: cast_nullable_to_non_nullable
as List<DiscloseCardRequest>,policy: null == policy ? _self.policy : policy // ignore: cast_nullable_to_non_nullable
as Policy,sharedDataWithOrganizationBefore: null == sharedDataWithOrganizationBefore ? _self.sharedDataWithOrganizationBefore : sharedDataWithOrganizationBefore // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

/// Create a copy of StartIssuanceResult
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


class StartIssuanceMissingAttributes extends StartIssuanceResult {
  const StartIssuanceMissingAttributes({required this.relyingParty, required this.originUrl, required final  LocalizedText requestPurpose, required this.sessionType, required final  List<MissingAttribute> missingAttributes, required this.sharedDataWithOrganizationBefore}): _requestPurpose = requestPurpose,_missingAttributes = missingAttributes,super._();
  

 final  Organization relyingParty;
 final  String originUrl;
 final  LocalizedText _requestPurpose;
 LocalizedText get requestPurpose {
  if (_requestPurpose is EqualUnmodifiableMapView) return _requestPurpose;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_requestPurpose);
}

 final  DisclosureSessionType sessionType;
 final  List<MissingAttribute> _missingAttributes;
 List<MissingAttribute> get missingAttributes {
  if (_missingAttributes is EqualUnmodifiableListView) return _missingAttributes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_missingAttributes);
}

 final  bool sharedDataWithOrganizationBefore;

/// Create a copy of StartIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StartIssuanceMissingAttributesCopyWith<StartIssuanceMissingAttributes> get copyWith => _$StartIssuanceMissingAttributesCopyWithImpl<StartIssuanceMissingAttributes>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartIssuanceMissingAttributes&&(identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty)&&(identical(other.originUrl, originUrl) || other.originUrl == originUrl)&&const DeepCollectionEquality().equals(other._requestPurpose, _requestPurpose)&&(identical(other.sessionType, sessionType) || other.sessionType == sessionType)&&const DeepCollectionEquality().equals(other._missingAttributes, _missingAttributes)&&(identical(other.sharedDataWithOrganizationBefore, sharedDataWithOrganizationBefore) || other.sharedDataWithOrganizationBefore == sharedDataWithOrganizationBefore));
}


@override
int get hashCode => Object.hash(runtimeType,relyingParty,originUrl,const DeepCollectionEquality().hash(_requestPurpose),sessionType,const DeepCollectionEquality().hash(_missingAttributes),sharedDataWithOrganizationBefore);

@override
String toString() {
  return 'StartIssuanceResult.missingAttributes(relyingParty: $relyingParty, originUrl: $originUrl, requestPurpose: $requestPurpose, sessionType: $sessionType, missingAttributes: $missingAttributes, sharedDataWithOrganizationBefore: $sharedDataWithOrganizationBefore)';
}


}

/// @nodoc
abstract mixin class $StartIssuanceMissingAttributesCopyWith<$Res> implements $StartIssuanceResultCopyWith<$Res> {
  factory $StartIssuanceMissingAttributesCopyWith(StartIssuanceMissingAttributes value, $Res Function(StartIssuanceMissingAttributes) _then) = _$StartIssuanceMissingAttributesCopyWithImpl;
@useResult
$Res call({
 Organization relyingParty, String originUrl, LocalizedText requestPurpose, DisclosureSessionType sessionType, List<MissingAttribute> missingAttributes, bool sharedDataWithOrganizationBefore
});


$OrganizationCopyWith<$Res> get relyingParty;

}
/// @nodoc
class _$StartIssuanceMissingAttributesCopyWithImpl<$Res>
    implements $StartIssuanceMissingAttributesCopyWith<$Res> {
  _$StartIssuanceMissingAttributesCopyWithImpl(this._self, this._then);

  final StartIssuanceMissingAttributes _self;
  final $Res Function(StartIssuanceMissingAttributes) _then;

/// Create a copy of StartIssuanceResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? relyingParty = null,Object? originUrl = null,Object? requestPurpose = null,Object? sessionType = null,Object? missingAttributes = null,Object? sharedDataWithOrganizationBefore = null,}) {
  return _then(StartIssuanceMissingAttributes(
relyingParty: null == relyingParty ? _self.relyingParty : relyingParty // ignore: cast_nullable_to_non_nullable
as Organization,originUrl: null == originUrl ? _self.originUrl : originUrl // ignore: cast_nullable_to_non_nullable
as String,requestPurpose: null == requestPurpose ? _self._requestPurpose : requestPurpose // ignore: cast_nullable_to_non_nullable
as LocalizedText,sessionType: null == sessionType ? _self.sessionType : sessionType // ignore: cast_nullable_to_non_nullable
as DisclosureSessionType,missingAttributes: null == missingAttributes ? _self._missingAttributes : missingAttributes // ignore: cast_nullable_to_non_nullable
as List<MissingAttribute>,sharedDataWithOrganizationBefore: null == sharedDataWithOrganizationBefore ? _self.sharedDataWithOrganizationBefore : sharedDataWithOrganizationBefore // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

/// Create a copy of StartIssuanceResult
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
