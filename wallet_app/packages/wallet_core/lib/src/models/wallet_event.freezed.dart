// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
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
  String get id;
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
            (identical(other.id, id) || other.id == id) &&
            (identical(other.dateTime, dateTime) || other.dateTime == dateTime));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, dateTime);

  @override
  String toString() {
    return 'WalletEvent(id: $id, dateTime: $dateTime)';
  }
}

/// @nodoc
abstract mixin class $WalletEventCopyWith<$Res> {
  factory $WalletEventCopyWith(WalletEvent value, $Res Function(WalletEvent) _then) = _$WalletEventCopyWithImpl;
  @useResult
  $Res call({String id, String dateTime});
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
    Object? id = null,
    Object? dateTime = null,
  }) {
    return _then(_self.copyWith(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      dateTime: null == dateTime
          ? _self.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [WalletEvent].
extension WalletEventPatterns on WalletEvent {
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

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletEvent_Disclosure value)? disclosure,
    TResult Function(WalletEvent_Issuance value)? issuance,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case WalletEvent_Disclosure() when disclosure != null:
        return disclosure(_that);
      case WalletEvent_Issuance() when issuance != null:
        return issuance(_that);
      case _:
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

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletEvent_Disclosure value) disclosure,
    required TResult Function(WalletEvent_Issuance value) issuance,
  }) {
    final _that = this;
    switch (_that) {
      case WalletEvent_Disclosure():
        return disclosure(_that);
      case WalletEvent_Issuance():
        return issuance(_that);
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

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletEvent_Disclosure value)? disclosure,
    TResult? Function(WalletEvent_Issuance value)? issuance,
  }) {
    final _that = this;
    switch (_that) {
      case WalletEvent_Disclosure() when disclosure != null:
        return disclosure(_that);
      case WalletEvent_Issuance() when issuance != null:
        return issuance(_that);
      case _:
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

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String id,
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<AttestationPresentation>? sharedAttestations,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)?
        disclosure,
    TResult Function(String id, String dateTime, AttestationPresentation attestation, bool renewed)? issuance,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case WalletEvent_Disclosure() when disclosure != null:
        return disclosure(_that.id, _that.dateTime, _that.relyingParty, _that.purpose, _that.sharedAttestations,
            _that.requestPolicy, _that.status, _that.typ);
      case WalletEvent_Issuance() when issuance != null:
        return issuance(_that.id, _that.dateTime, _that.attestation, _that.renewed);
      case _:
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

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String id,
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<AttestationPresentation>? sharedAttestations,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)
        disclosure,
    required TResult Function(String id, String dateTime, AttestationPresentation attestation, bool renewed) issuance,
  }) {
    final _that = this;
    switch (_that) {
      case WalletEvent_Disclosure():
        return disclosure(_that.id, _that.dateTime, _that.relyingParty, _that.purpose, _that.sharedAttestations,
            _that.requestPolicy, _that.status, _that.typ);
      case WalletEvent_Issuance():
        return issuance(_that.id, _that.dateTime, _that.attestation, _that.renewed);
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

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String id,
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<AttestationPresentation>? sharedAttestations,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)?
        disclosure,
    TResult? Function(String id, String dateTime, AttestationPresentation attestation, bool renewed)? issuance,
  }) {
    final _that = this;
    switch (_that) {
      case WalletEvent_Disclosure() when disclosure != null:
        return disclosure(_that.id, _that.dateTime, _that.relyingParty, _that.purpose, _that.sharedAttestations,
            _that.requestPolicy, _that.status, _that.typ);
      case WalletEvent_Issuance() when issuance != null:
        return issuance(_that.id, _that.dateTime, _that.attestation, _that.renewed);
      case _:
        return null;
    }
  }
}

/// @nodoc

class WalletEvent_Disclosure extends WalletEvent {
  const WalletEvent_Disclosure(
      {required this.id,
      required this.dateTime,
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
  final String id;
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
            (identical(other.id, id) || other.id == id) &&
            (identical(other.dateTime, dateTime) || other.dateTime == dateTime) &&
            (identical(other.relyingParty, relyingParty) || other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._purpose, _purpose) &&
            const DeepCollectionEquality().equals(other._sharedAttestations, _sharedAttestations) &&
            (identical(other.requestPolicy, requestPolicy) || other.requestPolicy == requestPolicy) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.typ, typ) || other.typ == typ));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      dateTime,
      relyingParty,
      const DeepCollectionEquality().hash(_purpose),
      const DeepCollectionEquality().hash(_sharedAttestations),
      requestPolicy,
      status,
      typ);

  @override
  String toString() {
    return 'WalletEvent.disclosure(id: $id, dateTime: $dateTime, relyingParty: $relyingParty, purpose: $purpose, sharedAttestations: $sharedAttestations, requestPolicy: $requestPolicy, status: $status, typ: $typ)';
  }
}

/// @nodoc
abstract mixin class $WalletEvent_DisclosureCopyWith<$Res> implements $WalletEventCopyWith<$Res> {
  factory $WalletEvent_DisclosureCopyWith(WalletEvent_Disclosure value, $Res Function(WalletEvent_Disclosure) _then) =
      _$WalletEvent_DisclosureCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String id,
      String dateTime,
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
    Object? id = null,
    Object? dateTime = null,
    Object? relyingParty = null,
    Object? purpose = null,
    Object? sharedAttestations = freezed,
    Object? requestPolicy = null,
    Object? status = null,
    Object? typ = null,
  }) {
    return _then(WalletEvent_Disclosure(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
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
  const WalletEvent_Issuance(
      {required this.id, required this.dateTime, required this.attestation, required this.renewed})
      : super._();

  @override
  final String id;
  @override
  final String dateTime;
  final AttestationPresentation attestation;
  final bool renewed;

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
            (identical(other.id, id) || other.id == id) &&
            (identical(other.dateTime, dateTime) || other.dateTime == dateTime) &&
            (identical(other.attestation, attestation) || other.attestation == attestation) &&
            (identical(other.renewed, renewed) || other.renewed == renewed));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, dateTime, attestation, renewed);

  @override
  String toString() {
    return 'WalletEvent.issuance(id: $id, dateTime: $dateTime, attestation: $attestation, renewed: $renewed)';
  }
}

/// @nodoc
abstract mixin class $WalletEvent_IssuanceCopyWith<$Res> implements $WalletEventCopyWith<$Res> {
  factory $WalletEvent_IssuanceCopyWith(WalletEvent_Issuance value, $Res Function(WalletEvent_Issuance) _then) =
      _$WalletEvent_IssuanceCopyWithImpl;
  @override
  @useResult
  $Res call({String id, String dateTime, AttestationPresentation attestation, bool renewed});
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
    Object? id = null,
    Object? dateTime = null,
    Object? attestation = null,
    Object? renewed = null,
  }) {
    return _then(WalletEvent_Issuance(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      dateTime: null == dateTime
          ? _self.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
      attestation: null == attestation
          ? _self.attestation
          : attestation // ignore: cast_nullable_to_non_nullable
              as AttestationPresentation,
      renewed: null == renewed
          ? _self.renewed
          : renewed // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

// dart format on
