// dart format width=80
// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'wallet_event.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$WalletEvent {
  String get dateTime;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $WalletEventCopyWith<WalletEvent> get copyWith =>
      _$WalletEventCopyWithImpl<WalletEvent>(this as WalletEvent, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is WalletEvent &&
            (identical(other.dateTime, dateTime) || other.dateTime == dateTime));
  }

  @override
  int get hashCode => Object.hash(runtimeType, dateTime);

  @override
  String toString() {
    return 'WalletEvent(dateTime: $dateTime)';
  }
}

/// @nodoc
abstract mixin class $WalletEventCopyWith<$Res> {
  factory $WalletEventCopyWith(WalletEvent value, $Res Function(WalletEvent) _then) = _$WalletEventCopyWithImpl;
  @useResult
  $Res call({String dateTime});
}

/// @nodoc
class _$WalletEventCopyWithImpl<$Res> implements $WalletEventCopyWith<$Res> {
  _$WalletEventCopyWithImpl(this._self, this._then);

  final WalletEvent _self;
  final $Res Function(WalletEvent) _then;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? dateTime = null,
  }) {
    return _then(_self.copyWith(
      dateTime: null == dateTime
          ? _self.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class WalletEvent_Disclosure extends WalletEvent {
  const WalletEvent_Disclosure(
      {required this.dateTime,
      required this.relyingParty,
      required final List<LocalizedString> purpose,
      final List<AttestationPresentation>? sharedAttestations,
      required this.requestPolicy,
      required this.status,
      required this.typ})
      : _purpose = purpose,
        _sharedAttestations = sharedAttestations,
        super._();

  @override
  final String dateTime;
  final Organization relyingParty;
  final List<LocalizedString> _purpose;
  List<LocalizedString> get purpose {
    if (_purpose is EqualUnmodifiableListView) return _purpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_purpose);
  }

  final List<AttestationPresentation>? _sharedAttestations;
  List<AttestationPresentation>? get sharedAttestations {
    final value = _sharedAttestations;
    if (value == null) return null;
    if (_sharedAttestations is EqualUnmodifiableListView) return _sharedAttestations;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  final RequestPolicy requestPolicy;
  final DisclosureStatus status;
  final DisclosureType typ;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $WalletEvent_DisclosureCopyWith<WalletEvent_Disclosure> get copyWith =>
      _$WalletEvent_DisclosureCopyWithImpl<WalletEvent_Disclosure>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is WalletEvent_Disclosure &&
            (identical(other.dateTime, dateTime) || other.dateTime == dateTime) &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._purpose, _purpose) &&
            const DeepCollectionEquality().equals(other._sharedAttestations, _sharedAttestations) &&
            (identical(other.requestPolicy, requestPolicy) || other.requestPolicy == requestPolicy) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.typ, typ) || other.typ == typ));
  }

  @override
  int get hashCode => Object.hash(runtimeType, dateTime, relyingParty, const DeepCollectionEquality().hash(_purpose),
      const DeepCollectionEquality().hash(_sharedAttestations), requestPolicy, status, typ);

  @override
  String toString() {
    return 'WalletEvent.disclosure(dateTime: $dateTime, relyingParty: $relyingParty, purpose: $purpose, sharedAttestations: $sharedAttestations, requestPolicy: $requestPolicy, status: $status, typ: $typ)';
  }
}

/// @nodoc
abstract mixin class $WalletEvent_DisclosureCopyWith<$Res> implements $WalletEventCopyWith<$Res> {
  factory $WalletEvent_DisclosureCopyWith(WalletEvent_Disclosure value, $Res Function(WalletEvent_Disclosure) _then) =
      _$WalletEvent_DisclosureCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String dateTime,
      Organization relyingParty,
      List<LocalizedString> purpose,
      List<AttestationPresentation>? sharedAttestations,
      RequestPolicy requestPolicy,
      DisclosureStatus status,
      DisclosureType typ});
}

/// @nodoc
class _$WalletEvent_DisclosureCopyWithImpl<$Res> implements $WalletEvent_DisclosureCopyWith<$Res> {
  _$WalletEvent_DisclosureCopyWithImpl(this._self, this._then);

  final WalletEvent_Disclosure _self;
  final $Res Function(WalletEvent_Disclosure) _then;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? dateTime = null,
    Object? relyingParty = null,
    Object? purpose = null,
    Object? sharedAttestations = freezed,
    Object? requestPolicy = null,
    Object? status = null,
    Object? typ = null,
  }) {
    return _then(WalletEvent_Disclosure(
      dateTime: null == dateTime
          ? _self.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
      relyingParty: null == relyingParty
          ? _self.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      purpose: null == purpose
          ? _self._purpose
          : purpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      sharedAttestations: freezed == sharedAttestations
          ? _self._sharedAttestations
          : sharedAttestations // ignore: cast_nullable_to_non_nullable
              as List<AttestationPresentation>?,
      requestPolicy: null == requestPolicy
          ? _self.requestPolicy
          : requestPolicy // ignore: cast_nullable_to_non_nullable
              as RequestPolicy,
      status: null == status
          ? _self.status
          : status // ignore: cast_nullable_to_non_nullable
              as DisclosureStatus,
      typ: null == typ
          ? _self.typ
          : typ // ignore: cast_nullable_to_non_nullable
              as DisclosureType,
    ));
  }
}

/// @nodoc

class WalletEvent_Issuance extends WalletEvent {
  const WalletEvent_Issuance({required this.dateTime, required this.attestation}) : super._();

  @override
  final String dateTime;
  final AttestationPresentation attestation;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $WalletEvent_IssuanceCopyWith<WalletEvent_Issuance> get copyWith =>
      _$WalletEvent_IssuanceCopyWithImpl<WalletEvent_Issuance>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is WalletEvent_Issuance &&
            (identical(other.dateTime, dateTime) || other.dateTime == dateTime) &&
            (identical(other.attestation, attestation) || other.attestation == attestation));
  }

  @override
  int get hashCode => Object.hash(runtimeType, dateTime, attestation);

  @override
  String toString() {
    return 'WalletEvent.issuance(dateTime: $dateTime, attestation: $attestation)';
  }
}

/// @nodoc
abstract mixin class $WalletEvent_IssuanceCopyWith<$Res> implements $WalletEventCopyWith<$Res> {
  factory $WalletEvent_IssuanceCopyWith(WalletEvent_Issuance value, $Res Function(WalletEvent_Issuance) _then) =
      _$WalletEvent_IssuanceCopyWithImpl;
  @override
  @useResult
  $Res call({String dateTime, AttestationPresentation attestation});
}

/// @nodoc
class _$WalletEvent_IssuanceCopyWithImpl<$Res> implements $WalletEvent_IssuanceCopyWith<$Res> {
  _$WalletEvent_IssuanceCopyWithImpl(this._self, this._then);

  final WalletEvent_Issuance _self;
  final $Res Function(WalletEvent_Issuance) _then;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? dateTime = null,
    Object? attestation = null,
  }) {
    return _then(WalletEvent_Issuance(
      dateTime: null == dateTime
          ? _self.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
      attestation: null == attestation
          ? _self.attestation
          : attestation // ignore: cast_nullable_to_non_nullable
              as AttestationPresentation,
    ));
  }
}

// dart format on
