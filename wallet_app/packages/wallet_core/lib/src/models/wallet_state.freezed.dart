// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'wallet_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$WalletState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState()';
}


}

/// @nodoc
class $WalletStateCopyWith<$Res>  {
$WalletStateCopyWith(WalletState _, $Res Function(WalletState) __);
}


/// Adds pattern-matching-related methods to [WalletState].
extension WalletStatePatterns on WalletState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WalletState_Blocked value)?  blocked,TResult Function( WalletState_Unregistered value)?  unregistered,TResult Function( WalletState_Locked value)?  locked,TResult Function( WalletState_Empty value)?  empty,TResult Function( WalletState_TransferPossible value)?  transferPossible,TResult Function( WalletState_Transferring value)?  transferring,TResult Function( WalletState_InDisclosureFlow value)?  inDisclosureFlow,TResult Function( WalletState_InIssuanceFlow value)?  inIssuanceFlow,TResult Function( WalletState_InPinChangeFlow value)?  inPinChangeFlow,TResult Function( WalletState_InPinRecoveryFlow value)?  inPinRecoveryFlow,TResult Function( WalletState_Ready value)?  ready,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WalletState_Blocked() when blocked != null:
return blocked(_that);case WalletState_Unregistered() when unregistered != null:
return unregistered(_that);case WalletState_Locked() when locked != null:
return locked(_that);case WalletState_Empty() when empty != null:
return empty(_that);case WalletState_TransferPossible() when transferPossible != null:
return transferPossible(_that);case WalletState_Transferring() when transferring != null:
return transferring(_that);case WalletState_InDisclosureFlow() when inDisclosureFlow != null:
return inDisclosureFlow(_that);case WalletState_InIssuanceFlow() when inIssuanceFlow != null:
return inIssuanceFlow(_that);case WalletState_InPinChangeFlow() when inPinChangeFlow != null:
return inPinChangeFlow(_that);case WalletState_InPinRecoveryFlow() when inPinRecoveryFlow != null:
return inPinRecoveryFlow(_that);case WalletState_Ready() when ready != null:
return ready(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WalletState_Blocked value)  blocked,required TResult Function( WalletState_Unregistered value)  unregistered,required TResult Function( WalletState_Locked value)  locked,required TResult Function( WalletState_Empty value)  empty,required TResult Function( WalletState_TransferPossible value)  transferPossible,required TResult Function( WalletState_Transferring value)  transferring,required TResult Function( WalletState_InDisclosureFlow value)  inDisclosureFlow,required TResult Function( WalletState_InIssuanceFlow value)  inIssuanceFlow,required TResult Function( WalletState_InPinChangeFlow value)  inPinChangeFlow,required TResult Function( WalletState_InPinRecoveryFlow value)  inPinRecoveryFlow,required TResult Function( WalletState_Ready value)  ready,}){
final _that = this;
switch (_that) {
case WalletState_Blocked():
return blocked(_that);case WalletState_Unregistered():
return unregistered(_that);case WalletState_Locked():
return locked(_that);case WalletState_Empty():
return empty(_that);case WalletState_TransferPossible():
return transferPossible(_that);case WalletState_Transferring():
return transferring(_that);case WalletState_InDisclosureFlow():
return inDisclosureFlow(_that);case WalletState_InIssuanceFlow():
return inIssuanceFlow(_that);case WalletState_InPinChangeFlow():
return inPinChangeFlow(_that);case WalletState_InPinRecoveryFlow():
return inPinRecoveryFlow(_that);case WalletState_Ready():
return ready(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WalletState_Blocked value)?  blocked,TResult? Function( WalletState_Unregistered value)?  unregistered,TResult? Function( WalletState_Locked value)?  locked,TResult? Function( WalletState_Empty value)?  empty,TResult? Function( WalletState_TransferPossible value)?  transferPossible,TResult? Function( WalletState_Transferring value)?  transferring,TResult? Function( WalletState_InDisclosureFlow value)?  inDisclosureFlow,TResult? Function( WalletState_InIssuanceFlow value)?  inIssuanceFlow,TResult? Function( WalletState_InPinChangeFlow value)?  inPinChangeFlow,TResult? Function( WalletState_InPinRecoveryFlow value)?  inPinRecoveryFlow,TResult? Function( WalletState_Ready value)?  ready,}){
final _that = this;
switch (_that) {
case WalletState_Blocked() when blocked != null:
return blocked(_that);case WalletState_Unregistered() when unregistered != null:
return unregistered(_that);case WalletState_Locked() when locked != null:
return locked(_that);case WalletState_Empty() when empty != null:
return empty(_that);case WalletState_TransferPossible() when transferPossible != null:
return transferPossible(_that);case WalletState_Transferring() when transferring != null:
return transferring(_that);case WalletState_InDisclosureFlow() when inDisclosureFlow != null:
return inDisclosureFlow(_that);case WalletState_InIssuanceFlow() when inIssuanceFlow != null:
return inIssuanceFlow(_that);case WalletState_InPinChangeFlow() when inPinChangeFlow != null:
return inPinChangeFlow(_that);case WalletState_InPinRecoveryFlow() when inPinRecoveryFlow != null:
return inPinRecoveryFlow(_that);case WalletState_Ready() when ready != null:
return ready(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( BlockedReason reason,  bool canRegisterNewAccount)?  blocked,TResult Function()?  unregistered,TResult Function( WalletState subState)?  locked,TResult Function()?  empty,TResult Function()?  transferPossible,TResult Function( TransferRole role)?  transferring,TResult Function()?  inDisclosureFlow,TResult Function()?  inIssuanceFlow,TResult Function()?  inPinChangeFlow,TResult Function()?  inPinRecoveryFlow,TResult Function()?  ready,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WalletState_Blocked() when blocked != null:
return blocked(_that.reason,_that.canRegisterNewAccount);case WalletState_Unregistered() when unregistered != null:
return unregistered();case WalletState_Locked() when locked != null:
return locked(_that.subState);case WalletState_Empty() when empty != null:
return empty();case WalletState_TransferPossible() when transferPossible != null:
return transferPossible();case WalletState_Transferring() when transferring != null:
return transferring(_that.role);case WalletState_InDisclosureFlow() when inDisclosureFlow != null:
return inDisclosureFlow();case WalletState_InIssuanceFlow() when inIssuanceFlow != null:
return inIssuanceFlow();case WalletState_InPinChangeFlow() when inPinChangeFlow != null:
return inPinChangeFlow();case WalletState_InPinRecoveryFlow() when inPinRecoveryFlow != null:
return inPinRecoveryFlow();case WalletState_Ready() when ready != null:
return ready();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( BlockedReason reason,  bool canRegisterNewAccount)  blocked,required TResult Function()  unregistered,required TResult Function( WalletState subState)  locked,required TResult Function()  empty,required TResult Function()  transferPossible,required TResult Function( TransferRole role)  transferring,required TResult Function()  inDisclosureFlow,required TResult Function()  inIssuanceFlow,required TResult Function()  inPinChangeFlow,required TResult Function()  inPinRecoveryFlow,required TResult Function()  ready,}) {final _that = this;
switch (_that) {
case WalletState_Blocked():
return blocked(_that.reason,_that.canRegisterNewAccount);case WalletState_Unregistered():
return unregistered();case WalletState_Locked():
return locked(_that.subState);case WalletState_Empty():
return empty();case WalletState_TransferPossible():
return transferPossible();case WalletState_Transferring():
return transferring(_that.role);case WalletState_InDisclosureFlow():
return inDisclosureFlow();case WalletState_InIssuanceFlow():
return inIssuanceFlow();case WalletState_InPinChangeFlow():
return inPinChangeFlow();case WalletState_InPinRecoveryFlow():
return inPinRecoveryFlow();case WalletState_Ready():
return ready();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( BlockedReason reason,  bool canRegisterNewAccount)?  blocked,TResult? Function()?  unregistered,TResult? Function( WalletState subState)?  locked,TResult? Function()?  empty,TResult? Function()?  transferPossible,TResult? Function( TransferRole role)?  transferring,TResult? Function()?  inDisclosureFlow,TResult? Function()?  inIssuanceFlow,TResult? Function()?  inPinChangeFlow,TResult? Function()?  inPinRecoveryFlow,TResult? Function()?  ready,}) {final _that = this;
switch (_that) {
case WalletState_Blocked() when blocked != null:
return blocked(_that.reason,_that.canRegisterNewAccount);case WalletState_Unregistered() when unregistered != null:
return unregistered();case WalletState_Locked() when locked != null:
return locked(_that.subState);case WalletState_Empty() when empty != null:
return empty();case WalletState_TransferPossible() when transferPossible != null:
return transferPossible();case WalletState_Transferring() when transferring != null:
return transferring(_that.role);case WalletState_InDisclosureFlow() when inDisclosureFlow != null:
return inDisclosureFlow();case WalletState_InIssuanceFlow() when inIssuanceFlow != null:
return inIssuanceFlow();case WalletState_InPinChangeFlow() when inPinChangeFlow != null:
return inPinChangeFlow();case WalletState_InPinRecoveryFlow() when inPinRecoveryFlow != null:
return inPinRecoveryFlow();case WalletState_Ready() when ready != null:
return ready();case _:
  return null;

}
}

}

/// @nodoc


class WalletState_Blocked extends WalletState {
  const WalletState_Blocked({required this.reason, required this.canRegisterNewAccount}): super._();
  

 final  BlockedReason reason;
 final  bool canRegisterNewAccount;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WalletState_BlockedCopyWith<WalletState_Blocked> get copyWith => _$WalletState_BlockedCopyWithImpl<WalletState_Blocked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Blocked&&(identical(other.reason, reason) || other.reason == reason)&&(identical(other.canRegisterNewAccount, canRegisterNewAccount) || other.canRegisterNewAccount == canRegisterNewAccount));
}


@override
int get hashCode => Object.hash(runtimeType,reason,canRegisterNewAccount);

@override
String toString() {
  return 'WalletState.blocked(reason: $reason, canRegisterNewAccount: $canRegisterNewAccount)';
}


}

/// @nodoc
abstract mixin class $WalletState_BlockedCopyWith<$Res> implements $WalletStateCopyWith<$Res> {
  factory $WalletState_BlockedCopyWith(WalletState_Blocked value, $Res Function(WalletState_Blocked) _then) = _$WalletState_BlockedCopyWithImpl;
@useResult
$Res call({
 BlockedReason reason, bool canRegisterNewAccount
});




}
/// @nodoc
class _$WalletState_BlockedCopyWithImpl<$Res>
    implements $WalletState_BlockedCopyWith<$Res> {
  _$WalletState_BlockedCopyWithImpl(this._self, this._then);

  final WalletState_Blocked _self;
  final $Res Function(WalletState_Blocked) _then;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,Object? canRegisterNewAccount = null,}) {
  return _then(WalletState_Blocked(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as BlockedReason,canRegisterNewAccount: null == canRegisterNewAccount ? _self.canRegisterNewAccount : canRegisterNewAccount // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class WalletState_Unregistered extends WalletState {
  const WalletState_Unregistered(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Unregistered);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.unregistered()';
}


}




/// @nodoc


class WalletState_Locked extends WalletState {
  const WalletState_Locked({required this.subState}): super._();
  

 final  WalletState subState;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WalletState_LockedCopyWith<WalletState_Locked> get copyWith => _$WalletState_LockedCopyWithImpl<WalletState_Locked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Locked&&(identical(other.subState, subState) || other.subState == subState));
}


@override
int get hashCode => Object.hash(runtimeType,subState);

@override
String toString() {
  return 'WalletState.locked(subState: $subState)';
}


}

/// @nodoc
abstract mixin class $WalletState_LockedCopyWith<$Res> implements $WalletStateCopyWith<$Res> {
  factory $WalletState_LockedCopyWith(WalletState_Locked value, $Res Function(WalletState_Locked) _then) = _$WalletState_LockedCopyWithImpl;
@useResult
$Res call({
 WalletState subState
});


$WalletStateCopyWith<$Res> get subState;

}
/// @nodoc
class _$WalletState_LockedCopyWithImpl<$Res>
    implements $WalletState_LockedCopyWith<$Res> {
  _$WalletState_LockedCopyWithImpl(this._self, this._then);

  final WalletState_Locked _self;
  final $Res Function(WalletState_Locked) _then;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? subState = null,}) {
  return _then(WalletState_Locked(
subState: null == subState ? _self.subState : subState // ignore: cast_nullable_to_non_nullable
as WalletState,
  ));
}

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WalletStateCopyWith<$Res> get subState {
  
  return $WalletStateCopyWith<$Res>(_self.subState, (value) {
    return _then(_self.copyWith(subState: value));
  });
}
}

/// @nodoc


class WalletState_Empty extends WalletState {
  const WalletState_Empty(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Empty);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.empty()';
}


}




/// @nodoc


class WalletState_TransferPossible extends WalletState {
  const WalletState_TransferPossible(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_TransferPossible);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.transferPossible()';
}


}




/// @nodoc


class WalletState_Transferring extends WalletState {
  const WalletState_Transferring({required this.role}): super._();
  

 final  TransferRole role;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WalletState_TransferringCopyWith<WalletState_Transferring> get copyWith => _$WalletState_TransferringCopyWithImpl<WalletState_Transferring>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Transferring&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,role);

@override
String toString() {
  return 'WalletState.transferring(role: $role)';
}


}

/// @nodoc
abstract mixin class $WalletState_TransferringCopyWith<$Res> implements $WalletStateCopyWith<$Res> {
  factory $WalletState_TransferringCopyWith(WalletState_Transferring value, $Res Function(WalletState_Transferring) _then) = _$WalletState_TransferringCopyWithImpl;
@useResult
$Res call({
 TransferRole role
});




}
/// @nodoc
class _$WalletState_TransferringCopyWithImpl<$Res>
    implements $WalletState_TransferringCopyWith<$Res> {
  _$WalletState_TransferringCopyWithImpl(this._self, this._then);

  final WalletState_Transferring _self;
  final $Res Function(WalletState_Transferring) _then;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? role = null,}) {
  return _then(WalletState_Transferring(
role: null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as TransferRole,
  ));
}


}

/// @nodoc


class WalletState_InDisclosureFlow extends WalletState {
  const WalletState_InDisclosureFlow(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_InDisclosureFlow);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.inDisclosureFlow()';
}


}




/// @nodoc


class WalletState_InIssuanceFlow extends WalletState {
  const WalletState_InIssuanceFlow(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_InIssuanceFlow);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.inIssuanceFlow()';
}


}




/// @nodoc


class WalletState_InPinChangeFlow extends WalletState {
  const WalletState_InPinChangeFlow(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_InPinChangeFlow);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.inPinChangeFlow()';
}


}




/// @nodoc


class WalletState_InPinRecoveryFlow extends WalletState {
  const WalletState_InPinRecoveryFlow(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_InPinRecoveryFlow);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.inPinRecoveryFlow()';
}


}




/// @nodoc


class WalletState_Ready extends WalletState {
  const WalletState_Ready(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Ready);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.ready()';
}


}




// dart format on
