// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'wallet_event.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$WalletEvent {
  String get dateTime => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)
        disclosure,
    required TResult Function(String dateTime, Card card) issuance,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)?
        disclosure,
    TResult? Function(String dateTime, Card card)? issuance,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)?
        disclosure,
    TResult Function(String dateTime, Card card)? issuance,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletEvent_Disclosure value) disclosure,
    required TResult Function(WalletEvent_Issuance value) issuance,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletEvent_Disclosure value)? disclosure,
    TResult? Function(WalletEvent_Issuance value)? issuance,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletEvent_Disclosure value)? disclosure,
    TResult Function(WalletEvent_Issuance value)? issuance,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $WalletEventCopyWith<WalletEvent> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $WalletEventCopyWith<$Res> {
  factory $WalletEventCopyWith(
          WalletEvent value, $Res Function(WalletEvent) then) =
      _$WalletEventCopyWithImpl<$Res, WalletEvent>;
  @useResult
  $Res call({String dateTime});
}

/// @nodoc
class _$WalletEventCopyWithImpl<$Res, $Val extends WalletEvent>
    implements $WalletEventCopyWith<$Res> {
  _$WalletEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? dateTime = null,
  }) {
    return _then(_value.copyWith(
      dateTime: null == dateTime
          ? _value.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$WalletEvent_DisclosureImplCopyWith<$Res>
    implements $WalletEventCopyWith<$Res> {
  factory _$$WalletEvent_DisclosureImplCopyWith(
          _$WalletEvent_DisclosureImpl value,
          $Res Function(_$WalletEvent_DisclosureImpl) then) =
      __$$WalletEvent_DisclosureImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String dateTime,
      Organization relyingParty,
      List<LocalizedString> purpose,
      List<DisclosureCard>? requestedCards,
      RequestPolicy requestPolicy,
      DisclosureStatus status,
      DisclosureType typ});
}

/// @nodoc
class __$$WalletEvent_DisclosureImplCopyWithImpl<$Res>
    extends _$WalletEventCopyWithImpl<$Res, _$WalletEvent_DisclosureImpl>
    implements _$$WalletEvent_DisclosureImplCopyWith<$Res> {
  __$$WalletEvent_DisclosureImplCopyWithImpl(
      _$WalletEvent_DisclosureImpl _value,
      $Res Function(_$WalletEvent_DisclosureImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? dateTime = null,
    Object? relyingParty = null,
    Object? purpose = null,
    Object? requestedCards = freezed,
    Object? requestPolicy = null,
    Object? status = null,
    Object? typ = null,
  }) {
    return _then(_$WalletEvent_DisclosureImpl(
      dateTime: null == dateTime
          ? _value.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      purpose: null == purpose
          ? _value._purpose
          : purpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestedCards: freezed == requestedCards
          ? _value._requestedCards
          : requestedCards // ignore: cast_nullable_to_non_nullable
              as List<DisclosureCard>?,
      requestPolicy: null == requestPolicy
          ? _value.requestPolicy
          : requestPolicy // ignore: cast_nullable_to_non_nullable
              as RequestPolicy,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as DisclosureStatus,
      typ: null == typ
          ? _value.typ
          : typ // ignore: cast_nullable_to_non_nullable
              as DisclosureType,
    ));
  }
}

/// @nodoc

class _$WalletEvent_DisclosureImpl extends WalletEvent_Disclosure {
  const _$WalletEvent_DisclosureImpl(
      {required this.dateTime,
      required this.relyingParty,
      required final List<LocalizedString> purpose,
      final List<DisclosureCard>? requestedCards,
      required this.requestPolicy,
      required this.status,
      required this.typ})
      : _purpose = purpose,
        _requestedCards = requestedCards,
        super._();

  @override
  final String dateTime;
  @override
  final Organization relyingParty;
  final List<LocalizedString> _purpose;
  @override
  List<LocalizedString> get purpose {
    if (_purpose is EqualUnmodifiableListView) return _purpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_purpose);
  }

  final List<DisclosureCard>? _requestedCards;
  @override
  List<DisclosureCard>? get requestedCards {
    final value = _requestedCards;
    if (value == null) return null;
    if (_requestedCards is EqualUnmodifiableListView) return _requestedCards;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  @override
  final RequestPolicy requestPolicy;
  @override
  final DisclosureStatus status;
  @override
  final DisclosureType typ;

  @override
  String toString() {
    return 'WalletEvent.disclosure(dateTime: $dateTime, relyingParty: $relyingParty, purpose: $purpose, requestedCards: $requestedCards, requestPolicy: $requestPolicy, status: $status, typ: $typ)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletEvent_DisclosureImpl &&
            (identical(other.dateTime, dateTime) ||
                other.dateTime == dateTime) &&
            (identical(other.relyingParty, relyingParty) ||
                other.relyingParty == relyingParty) &&
            const DeepCollectionEquality().equals(other._purpose, _purpose) &&
            const DeepCollectionEquality()
                .equals(other._requestedCards, _requestedCards) &&
            (identical(other.requestPolicy, requestPolicy) ||
                other.requestPolicy == requestPolicy) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.typ, typ) || other.typ == typ));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      dateTime,
      relyingParty,
      const DeepCollectionEquality().hash(_purpose),
      const DeepCollectionEquality().hash(_requestedCards),
      requestPolicy,
      status,
      typ);

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletEvent_DisclosureImplCopyWith<_$WalletEvent_DisclosureImpl>
      get copyWith => __$$WalletEvent_DisclosureImplCopyWithImpl<
          _$WalletEvent_DisclosureImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)
        disclosure,
    required TResult Function(String dateTime, Card card) issuance,
  }) {
    return disclosure(dateTime, relyingParty, purpose, requestedCards,
        requestPolicy, status, typ);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)?
        disclosure,
    TResult? Function(String dateTime, Card card)? issuance,
  }) {
    return disclosure?.call(dateTime, relyingParty, purpose, requestedCards,
        requestPolicy, status, typ);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)?
        disclosure,
    TResult Function(String dateTime, Card card)? issuance,
    required TResult orElse(),
  }) {
    if (disclosure != null) {
      return disclosure(dateTime, relyingParty, purpose, requestedCards,
          requestPolicy, status, typ);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletEvent_Disclosure value) disclosure,
    required TResult Function(WalletEvent_Issuance value) issuance,
  }) {
    return disclosure(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletEvent_Disclosure value)? disclosure,
    TResult? Function(WalletEvent_Issuance value)? issuance,
  }) {
    return disclosure?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletEvent_Disclosure value)? disclosure,
    TResult Function(WalletEvent_Issuance value)? issuance,
    required TResult orElse(),
  }) {
    if (disclosure != null) {
      return disclosure(this);
    }
    return orElse();
  }
}

abstract class WalletEvent_Disclosure extends WalletEvent {
  const factory WalletEvent_Disclosure(
      {required final String dateTime,
      required final Organization relyingParty,
      required final List<LocalizedString> purpose,
      final List<DisclosureCard>? requestedCards,
      required final RequestPolicy requestPolicy,
      required final DisclosureStatus status,
      required final DisclosureType typ}) = _$WalletEvent_DisclosureImpl;
  const WalletEvent_Disclosure._() : super._();

  @override
  String get dateTime;
  Organization get relyingParty;
  List<LocalizedString> get purpose;
  List<DisclosureCard>? get requestedCards;
  RequestPolicy get requestPolicy;
  DisclosureStatus get status;
  DisclosureType get typ;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WalletEvent_DisclosureImplCopyWith<_$WalletEvent_DisclosureImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$WalletEvent_IssuanceImplCopyWith<$Res>
    implements $WalletEventCopyWith<$Res> {
  factory _$$WalletEvent_IssuanceImplCopyWith(_$WalletEvent_IssuanceImpl value,
          $Res Function(_$WalletEvent_IssuanceImpl) then) =
      __$$WalletEvent_IssuanceImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String dateTime, Card card});
}

/// @nodoc
class __$$WalletEvent_IssuanceImplCopyWithImpl<$Res>
    extends _$WalletEventCopyWithImpl<$Res, _$WalletEvent_IssuanceImpl>
    implements _$$WalletEvent_IssuanceImplCopyWith<$Res> {
  __$$WalletEvent_IssuanceImplCopyWithImpl(_$WalletEvent_IssuanceImpl _value,
      $Res Function(_$WalletEvent_IssuanceImpl) _then)
      : super(_value, _then);

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? dateTime = null,
    Object? card = null,
  }) {
    return _then(_$WalletEvent_IssuanceImpl(
      dateTime: null == dateTime
          ? _value.dateTime
          : dateTime // ignore: cast_nullable_to_non_nullable
              as String,
      card: null == card
          ? _value.card
          : card // ignore: cast_nullable_to_non_nullable
              as Card,
    ));
  }
}

/// @nodoc

class _$WalletEvent_IssuanceImpl extends WalletEvent_Issuance {
  const _$WalletEvent_IssuanceImpl({required this.dateTime, required this.card})
      : super._();

  @override
  final String dateTime;
  @override
  final Card card;

  @override
  String toString() {
    return 'WalletEvent.issuance(dateTime: $dateTime, card: $card)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WalletEvent_IssuanceImpl &&
            (identical(other.dateTime, dateTime) ||
                other.dateTime == dateTime) &&
            (identical(other.card, card) || other.card == card));
  }

  @override
  int get hashCode => Object.hash(runtimeType, dateTime, card);

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$WalletEvent_IssuanceImplCopyWith<_$WalletEvent_IssuanceImpl>
      get copyWith =>
          __$$WalletEvent_IssuanceImplCopyWithImpl<_$WalletEvent_IssuanceImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)
        disclosure,
    required TResult Function(String dateTime, Card card) issuance,
  }) {
    return issuance(dateTime, card);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)?
        disclosure,
    TResult? Function(String dateTime, Card card)? issuance,
  }) {
    return issuance?.call(dateTime, card);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String dateTime,
            Organization relyingParty,
            List<LocalizedString> purpose,
            List<DisclosureCard>? requestedCards,
            RequestPolicy requestPolicy,
            DisclosureStatus status,
            DisclosureType typ)?
        disclosure,
    TResult Function(String dateTime, Card card)? issuance,
    required TResult orElse(),
  }) {
    if (issuance != null) {
      return issuance(dateTime, card);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(WalletEvent_Disclosure value) disclosure,
    required TResult Function(WalletEvent_Issuance value) issuance,
  }) {
    return issuance(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(WalletEvent_Disclosure value)? disclosure,
    TResult? Function(WalletEvent_Issuance value)? issuance,
  }) {
    return issuance?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(WalletEvent_Disclosure value)? disclosure,
    TResult Function(WalletEvent_Issuance value)? issuance,
    required TResult orElse(),
  }) {
    if (issuance != null) {
      return issuance(this);
    }
    return orElse();
  }
}

abstract class WalletEvent_Issuance extends WalletEvent {
  const factory WalletEvent_Issuance(
      {required final String dateTime,
      required final Card card}) = _$WalletEvent_IssuanceImpl;
  const WalletEvent_Issuance._() : super._();

  @override
  String get dateTime;
  Card get card;

  /// Create a copy of WalletEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WalletEvent_IssuanceImplCopyWith<_$WalletEvent_IssuanceImpl>
      get copyWith => throw _privateConstructorUsedError;
}
