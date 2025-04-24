// dart format width=80
// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'disclosure.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$AcceptDisclosureResult {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is AcceptDisclosureResult);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'AcceptDisclosureResult()';
  }
}

/// @nodoc
class $AcceptDisclosureResultCopyWith<$Res> {
  $AcceptDisclosureResultCopyWith(AcceptDisclosureResult _, $Res Function(AcceptDisclosureResult) __);
}

/// @nodoc

class AcceptDisclosureResult_Ok extends AcceptDisclosureResult {
  const AcceptDisclosureResult_Ok({this.returnUrl}) : super._();

  final String? returnUrl;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AcceptDisclosureResult_OkCopyWith<AcceptDisclosureResult_Ok> get copyWith =>
      _$AcceptDisclosureResult_OkCopyWithImpl<AcceptDisclosureResult_Ok>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AcceptDisclosureResult_Ok &&
            (identical(other.returnUrl, returnUrl) || other.returnUrl == returnUrl));
  }

  @override
  int get hashCode => Object.hash(runtimeType, returnUrl);

  @override
  String toString() {
    return 'AcceptDisclosureResult.ok(returnUrl: $returnUrl)';
  }
}

/// @nodoc
abstract mixin class $AcceptDisclosureResult_OkCopyWith<$Res> implements $AcceptDisclosureResultCopyWith<$Res> {
  factory $AcceptDisclosureResult_OkCopyWith(
          AcceptDisclosureResult_Ok value, $Res Function(AcceptDisclosureResult_Ok) _then) =
      _$AcceptDisclosureResult_OkCopyWithImpl;
  @useResult
  $Res call({String? returnUrl});
}

/// @nodoc
class _$AcceptDisclosureResult_OkCopyWithImpl<$Res> implements $AcceptDisclosureResult_OkCopyWith<$Res> {
  _$AcceptDisclosureResult_OkCopyWithImpl(this._self, this._then);

  final AcceptDisclosureResult_Ok _self;
  final $Res Function(AcceptDisclosureResult_Ok) _then;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? returnUrl = freezed,
  }) {
    return _then(AcceptDisclosureResult_Ok(
      returnUrl: freezed == returnUrl
          ? _self.returnUrl
          : returnUrl // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc

class AcceptDisclosureResult_InstructionError extends AcceptDisclosureResult {
  const AcceptDisclosureResult_InstructionError({required this.error}) : super._();

  final WalletInstructionError error;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AcceptDisclosureResult_InstructionErrorCopyWith<AcceptDisclosureResult_InstructionError> get copyWith =>
      _$AcceptDisclosureResult_InstructionErrorCopyWithImpl<AcceptDisclosureResult_InstructionError>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AcceptDisclosureResult_InstructionError &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  @override
  String toString() {
    return 'AcceptDisclosureResult.instructionError(error: $error)';
  }
}

/// @nodoc
abstract mixin class $AcceptDisclosureResult_InstructionErrorCopyWith<$Res>
    implements $AcceptDisclosureResultCopyWith<$Res> {
  factory $AcceptDisclosureResult_InstructionErrorCopyWith(
          AcceptDisclosureResult_InstructionError value, $Res Function(AcceptDisclosureResult_InstructionError) _then) =
      _$AcceptDisclosureResult_InstructionErrorCopyWithImpl;
  @useResult
  $Res call({WalletInstructionError error});

  $WalletInstructionErrorCopyWith<$Res> get error;
}

/// @nodoc
class _$AcceptDisclosureResult_InstructionErrorCopyWithImpl<$Res>
    implements $AcceptDisclosureResult_InstructionErrorCopyWith<$Res> {
  _$AcceptDisclosureResult_InstructionErrorCopyWithImpl(this._self, this._then);

  final AcceptDisclosureResult_InstructionError _self;
  final $Res Function(AcceptDisclosureResult_InstructionError) _then;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? error = null,
  }) {
    return _then(AcceptDisclosureResult_InstructionError(
      error: null == error
          ? _self.error
          : error // ignore: cast_nullable_to_non_nullable
              as WalletInstructionError,
    ));
  }

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $WalletInstructionErrorCopyWith<$Res> get error {
    return $WalletInstructionErrorCopyWith<$Res>(_self.error, (value) {
      return _then(_self.copyWith(error: value));
    });
  }
}

/// @nodoc
mixin _$StartDisclosureResult {
  Organization get relyingParty;
  bool get sharedDataWithRelyingPartyBefore;
  DisclosureSessionType get sessionType;
  List<LocalizedString> get requestPurpose;
  String get requestOriginBaseUrl;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $StartDisclosureResultCopyWith<StartDisclosureResult> get copyWith =>
      _$StartDisclosureResultCopyWithImpl<StartDisclosureResult>(this as StartDisclosureResult, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is StartDisclosureResult &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            (identical(other.sharedDataWithRelyingPartyBefore, sharedDataWithRelyingPartyBefore) ||
                other.sharedDataWithRelyingPartyBefore == sharedDataWithRelyingPartyBefore) &&
            (identical(other.sessionType, sessionType) || other.sessionType == sessionType) &&
            const DeepCollectionEquality().equals(other.requestPurpose, requestPurpose) &&
            (identical(other.requestOriginBaseUrl, requestOriginBaseUrl) ||
                other.requestOriginBaseUrl == requestOriginBaseUrl));
  }

  @override
  int get hashCode => Object.hash(runtimeType, relyingParty, sharedDataWithRelyingPartyBefore, sessionType,
      const DeepCollectionEquality().hash(requestPurpose), requestOriginBaseUrl);

  @override
  String toString() {
    return 'StartDisclosureResult(relyingParty: $relyingParty, sharedDataWithRelyingPartyBefore: $sharedDataWithRelyingPartyBefore, sessionType: $sessionType, requestPurpose: $requestPurpose, requestOriginBaseUrl: $requestOriginBaseUrl)';
  }
}

/// @nodoc
abstract mixin class $StartDisclosureResultCopyWith<$Res> {
  factory $StartDisclosureResultCopyWith(StartDisclosureResult value, $Res Function(StartDisclosureResult) _then) =
      _$StartDisclosureResultCopyWithImpl;
  @useResult
  $Res call(
      {Organization relyingParty,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl});
}

/// @nodoc
class _$StartDisclosureResultCopyWithImpl<$Res> implements $StartDisclosureResultCopyWith<$Res> {
  _$StartDisclosureResultCopyWithImpl(this._self, this._then);

  final StartDisclosureResult _self;
  final $Res Function(StartDisclosureResult) _then;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
  }) {
    return _then(_self.copyWith(
      relyingParty: null == relyingParty
          ? _self.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _self.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _self.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _self.requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _self.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class StartDisclosureResult_Request extends StartDisclosureResult {
  const StartDisclosureResult_Request(
      {required this.relyingParty,
      required this.policy,
      required final List<Attestation> requestedAttestations,
      required this.sharedDataWithRelyingPartyBefore,
      required this.sessionType,
      required final List<LocalizedString> requestPurpose,
      required this.requestOriginBaseUrl,
      required this.requestType})
      : _requestedAttestations = requestedAttestations,
        _requestPurpose = requestPurpose,
        super._();

  @override
  final Organization relyingParty;
  final RequestPolicy policy;
  final List<Attestation> _requestedAttestations;
  List<Attestation> get requestedAttestations {
    if (_requestedAttestations is EqualUnmodifiableListView) return _requestedAttestations;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestedAttestations);
  }

  @override
  final bool sharedDataWithRelyingPartyBefore;
  @override
  final DisclosureSessionType sessionType;
  final List<LocalizedString> _requestPurpose;
  @override
  List<LocalizedString> get requestPurpose {
    if (_requestPurpose is EqualUnmodifiableListView) return _requestPurpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestPurpose);
  }

  @override
  final String requestOriginBaseUrl;
  final DisclosureType requestType;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $StartDisclosureResult_RequestCopyWith<StartDisclosureResult_Request> get copyWith =>
      _$StartDisclosureResult_RequestCopyWithImpl<StartDisclosureResult_Request>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is StartDisclosureResult_Request &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            (identical(other.policy, policy) || other.policy == policy) &&
            const DeepCollectionEquality().equals(other._requestedAttestations, _requestedAttestations) &&
            (identical(other.sharedDataWithRelyingPartyBefore, sharedDataWithRelyingPartyBefore) ||
                other.sharedDataWithRelyingPartyBefore == sharedDataWithRelyingPartyBefore) &&
            (identical(other.sessionType, sessionType) || other.sessionType == sessionType) &&
            const DeepCollectionEquality().equals(other._requestPurpose, _requestPurpose) &&
            (identical(other.requestOriginBaseUrl, requestOriginBaseUrl) ||
                other.requestOriginBaseUrl == requestOriginBaseUrl) &&
            (identical(other.requestType, requestType) || other.requestType == requestType));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      relyingParty,
      policy,
      const DeepCollectionEquality().hash(_requestedAttestations),
      sharedDataWithRelyingPartyBefore,
      sessionType,
      const DeepCollectionEquality().hash(_requestPurpose),
      requestOriginBaseUrl,
      requestType);

  @override
  String toString() {
    return 'StartDisclosureResult.request(relyingParty: $relyingParty, policy: $policy, requestedAttestations: $requestedAttestations, sharedDataWithRelyingPartyBefore: $sharedDataWithRelyingPartyBefore, sessionType: $sessionType, requestPurpose: $requestPurpose, requestOriginBaseUrl: $requestOriginBaseUrl, requestType: $requestType)';
  }
}

/// @nodoc
abstract mixin class $StartDisclosureResult_RequestCopyWith<$Res> implements $StartDisclosureResultCopyWith<$Res> {
  factory $StartDisclosureResult_RequestCopyWith(
          StartDisclosureResult_Request value, $Res Function(StartDisclosureResult_Request) _then) =
      _$StartDisclosureResult_RequestCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Organization relyingParty,
      RequestPolicy policy,
      List<Attestation> requestedAttestations,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl,
      DisclosureType requestType});
}

/// @nodoc
class _$StartDisclosureResult_RequestCopyWithImpl<$Res> implements $StartDisclosureResult_RequestCopyWith<$Res> {
  _$StartDisclosureResult_RequestCopyWithImpl(this._self, this._then);

  final StartDisclosureResult_Request _self;
  final $Res Function(StartDisclosureResult_Request) _then;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? relyingParty = null,
    Object? policy = null,
    Object? requestedAttestations = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
    Object? requestType = null,
  }) {
    return _then(StartDisclosureResult_Request(
      relyingParty: null == relyingParty
          ? _self.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      policy: null == policy
          ? _self.policy
          : policy // ignore: cast_nullable_to_non_nullable
              as RequestPolicy,
      requestedAttestations: null == requestedAttestations
          ? _self._requestedAttestations
          : requestedAttestations // ignore: cast_nullable_to_non_nullable
              as List<Attestation>,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _self.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _self.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _self._requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _self.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
      requestType: null == requestType
          ? _self.requestType
          : requestType // ignore: cast_nullable_to_non_nullable
              as DisclosureType,
    ));
  }
}

/// @nodoc

class StartDisclosureResult_RequestAttributesMissing extends StartDisclosureResult {
  const StartDisclosureResult_RequestAttributesMissing(
      {required this.relyingParty,
      required final List<MissingAttribute> missingAttributes,
      required this.sharedDataWithRelyingPartyBefore,
      required this.sessionType,
      required final List<LocalizedString> requestPurpose,
      required this.requestOriginBaseUrl})
      : _missingAttributes = missingAttributes,
        _requestPurpose = requestPurpose,
        super._();

  @override
  final Organization relyingParty;
  final List<MissingAttribute> _missingAttributes;
  List<MissingAttribute> get missingAttributes {
    if (_missingAttributes is EqualUnmodifiableListView) return _missingAttributes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_missingAttributes);
  }

  @override
  final bool sharedDataWithRelyingPartyBefore;
  @override
  final DisclosureSessionType sessionType;
  final List<LocalizedString> _requestPurpose;
  @override
  List<LocalizedString> get requestPurpose {
    if (_requestPurpose is EqualUnmodifiableListView) return _requestPurpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestPurpose);
  }

  @override
  final String requestOriginBaseUrl;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $StartDisclosureResult_RequestAttributesMissingCopyWith<StartDisclosureResult_RequestAttributesMissing>
      get copyWith =>
          _$StartDisclosureResult_RequestAttributesMissingCopyWithImpl<StartDisclosureResult_RequestAttributesMissing>(
              this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is StartDisclosureResult_RequestAttributesMissing &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._missingAttributes, _missingAttributes) &&
            (identical(other.sharedDataWithRelyingPartyBefore, sharedDataWithRelyingPartyBefore) ||
                other.sharedDataWithRelyingPartyBefore == sharedDataWithRelyingPartyBefore) &&
            (identical(other.sessionType, sessionType) || other.sessionType == sessionType) &&
            const DeepCollectionEquality().equals(other._requestPurpose, _requestPurpose) &&
            (identical(other.requestOriginBaseUrl, requestOriginBaseUrl) ||
                other.requestOriginBaseUrl == requestOriginBaseUrl));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      relyingParty,
      const DeepCollectionEquality().hash(_missingAttributes),
      sharedDataWithRelyingPartyBefore,
      sessionType,
      const DeepCollectionEquality().hash(_requestPurpose),
      requestOriginBaseUrl);

  @override
  String toString() {
    return 'StartDisclosureResult.requestAttributesMissing(relyingParty: $relyingParty, missingAttributes: $missingAttributes, sharedDataWithRelyingPartyBefore: $sharedDataWithRelyingPartyBefore, sessionType: $sessionType, requestPurpose: $requestPurpose, requestOriginBaseUrl: $requestOriginBaseUrl)';
  }
}

/// @nodoc
abstract mixin class $StartDisclosureResult_RequestAttributesMissingCopyWith<$Res>
    implements $StartDisclosureResultCopyWith<$Res> {
  factory $StartDisclosureResult_RequestAttributesMissingCopyWith(StartDisclosureResult_RequestAttributesMissing value,
          $Res Function(StartDisclosureResult_RequestAttributesMissing) _then) =
      _$StartDisclosureResult_RequestAttributesMissingCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Organization relyingParty,
      List<MissingAttribute> missingAttributes,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl});
}

/// @nodoc
class _$StartDisclosureResult_RequestAttributesMissingCopyWithImpl<$Res>
    implements $StartDisclosureResult_RequestAttributesMissingCopyWith<$Res> {
  _$StartDisclosureResult_RequestAttributesMissingCopyWithImpl(this._self, this._then);

  final StartDisclosureResult_RequestAttributesMissing _self;
  final $Res Function(StartDisclosureResult_RequestAttributesMissing) _then;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? relyingParty = null,
    Object? missingAttributes = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
  }) {
    return _then(StartDisclosureResult_RequestAttributesMissing(
      relyingParty: null == relyingParty
          ? _self.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      missingAttributes: null == missingAttributes
          ? _self._missingAttributes
          : missingAttributes // ignore: cast_nullable_to_non_nullable
              as List<MissingAttribute>,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _self.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _self.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _self._requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _self.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

// dart format on
