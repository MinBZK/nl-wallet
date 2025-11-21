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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WalletState_Ready value)?  ready,TResult Function( WalletState_Registration value)?  registration,TResult Function( WalletState_Empty value)?  empty,TResult Function( WalletState_Locked value)?  locked,TResult Function( WalletState_TransferPossible value)?  transferPossible,TResult Function( WalletState_Transferring value)?  transferring,TResult Function( WalletState_Disclosure value)?  disclosure,TResult Function( WalletState_Issuance value)?  issuance,TResult Function( WalletState_PinChange value)?  pinChange,TResult Function( WalletState_PinRecovery value)?  pinRecovery,TResult Function( WalletState_WalletBlocked value)?  walletBlocked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WalletState_Ready() when ready != null:
return ready(_that);case WalletState_Registration() when registration != null:
return registration(_that);case WalletState_Empty() when empty != null:
return empty(_that);case WalletState_Locked() when locked != null:
return locked(_that);case WalletState_TransferPossible() when transferPossible != null:
return transferPossible(_that);case WalletState_Transferring() when transferring != null:
return transferring(_that);case WalletState_Disclosure() when disclosure != null:
return disclosure(_that);case WalletState_Issuance() when issuance != null:
return issuance(_that);case WalletState_PinChange() when pinChange != null:
return pinChange(_that);case WalletState_PinRecovery() when pinRecovery != null:
return pinRecovery(_that);case WalletState_WalletBlocked() when walletBlocked != null:
return walletBlocked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WalletState_Ready value)  ready,required TResult Function( WalletState_Registration value)  registration,required TResult Function( WalletState_Empty value)  empty,required TResult Function( WalletState_Locked value)  locked,required TResult Function( WalletState_TransferPossible value)  transferPossible,required TResult Function( WalletState_Transferring value)  transferring,required TResult Function( WalletState_Disclosure value)  disclosure,required TResult Function( WalletState_Issuance value)  issuance,required TResult Function( WalletState_PinChange value)  pinChange,required TResult Function( WalletState_PinRecovery value)  pinRecovery,required TResult Function( WalletState_WalletBlocked value)  walletBlocked,}){
final _that = this;
switch (_that) {
case WalletState_Ready():
return ready(_that);case WalletState_Registration():
return registration(_that);case WalletState_Empty():
return empty(_that);case WalletState_Locked():
return locked(_that);case WalletState_TransferPossible():
return transferPossible(_that);case WalletState_Transferring():
return transferring(_that);case WalletState_Disclosure():
return disclosure(_that);case WalletState_Issuance():
return issuance(_that);case WalletState_PinChange():
return pinChange(_that);case WalletState_PinRecovery():
return pinRecovery(_that);case WalletState_WalletBlocked():
return walletBlocked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WalletState_Ready value)?  ready,TResult? Function( WalletState_Registration value)?  registration,TResult? Function( WalletState_Empty value)?  empty,TResult? Function( WalletState_Locked value)?  locked,TResult? Function( WalletState_TransferPossible value)?  transferPossible,TResult? Function( WalletState_Transferring value)?  transferring,TResult? Function( WalletState_Disclosure value)?  disclosure,TResult? Function( WalletState_Issuance value)?  issuance,TResult? Function( WalletState_PinChange value)?  pinChange,TResult? Function( WalletState_PinRecovery value)?  pinRecovery,TResult? Function( WalletState_WalletBlocked value)?  walletBlocked,}){
final _that = this;
switch (_that) {
case WalletState_Ready() when ready != null:
return ready(_that);case WalletState_Registration() when registration != null:
return registration(_that);case WalletState_Empty() when empty != null:
return empty(_that);case WalletState_Locked() when locked != null:
return locked(_that);case WalletState_TransferPossible() when transferPossible != null:
return transferPossible(_that);case WalletState_Transferring() when transferring != null:
return transferring(_that);case WalletState_Disclosure() when disclosure != null:
return disclosure(_that);case WalletState_Issuance() when issuance != null:
return issuance(_that);case WalletState_PinChange() when pinChange != null:
return pinChange(_that);case WalletState_PinRecovery() when pinRecovery != null:
return pinRecovery(_that);case WalletState_WalletBlocked() when walletBlocked != null:
return walletBlocked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  ready,TResult Function()?  registration,TResult Function()?  empty,TResult Function( WalletState subState)?  locked,TResult Function()?  transferPossible,TResult Function( WalletTransferRole role)?  transferring,TResult Function()?  disclosure,TResult Function( bool pid)?  issuance,TResult Function()?  pinChange,TResult Function()?  pinRecovery,TResult Function( WalletBlockedReason reason)?  walletBlocked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WalletState_Ready() when ready != null:
return ready();case WalletState_Registration() when registration != null:
return registration();case WalletState_Empty() when empty != null:
return empty();case WalletState_Locked() when locked != null:
return locked(_that.subState);case WalletState_TransferPossible() when transferPossible != null:
return transferPossible();case WalletState_Transferring() when transferring != null:
return transferring(_that.role);case WalletState_Disclosure() when disclosure != null:
return disclosure();case WalletState_Issuance() when issuance != null:
return issuance(_that.pid);case WalletState_PinChange() when pinChange != null:
return pinChange();case WalletState_PinRecovery() when pinRecovery != null:
return pinRecovery();case WalletState_WalletBlocked() when walletBlocked != null:
return walletBlocked(_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  ready,required TResult Function()  registration,required TResult Function()  empty,required TResult Function( WalletState subState)  locked,required TResult Function()  transferPossible,required TResult Function( WalletTransferRole role)  transferring,required TResult Function()  disclosure,required TResult Function( bool pid)  issuance,required TResult Function()  pinChange,required TResult Function()  pinRecovery,required TResult Function( WalletBlockedReason reason)  walletBlocked,}) {final _that = this;
switch (_that) {
case WalletState_Ready():
return ready();case WalletState_Registration():
return registration();case WalletState_Empty():
return empty();case WalletState_Locked():
return locked(_that.subState);case WalletState_TransferPossible():
return transferPossible();case WalletState_Transferring():
return transferring(_that.role);case WalletState_Disclosure():
return disclosure();case WalletState_Issuance():
return issuance(_that.pid);case WalletState_PinChange():
return pinChange();case WalletState_PinRecovery():
return pinRecovery();case WalletState_WalletBlocked():
return walletBlocked(_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  ready,TResult? Function()?  registration,TResult? Function()?  empty,TResult? Function( WalletState subState)?  locked,TResult? Function()?  transferPossible,TResult? Function( WalletTransferRole role)?  transferring,TResult? Function()?  disclosure,TResult? Function( bool pid)?  issuance,TResult? Function()?  pinChange,TResult? Function()?  pinRecovery,TResult? Function( WalletBlockedReason reason)?  walletBlocked,}) {final _that = this;
switch (_that) {
case WalletState_Ready() when ready != null:
return ready();case WalletState_Registration() when registration != null:
return registration();case WalletState_Empty() when empty != null:
return empty();case WalletState_Locked() when locked != null:
return locked(_that.subState);case WalletState_TransferPossible() when transferPossible != null:
return transferPossible();case WalletState_Transferring() when transferring != null:
return transferring(_that.role);case WalletState_Disclosure() when disclosure != null:
return disclosure();case WalletState_Issuance() when issuance != null:
return issuance(_that.pid);case WalletState_PinChange() when pinChange != null:
return pinChange();case WalletState_PinRecovery() when pinRecovery != null:
return pinRecovery();case WalletState_WalletBlocked() when walletBlocked != null:
return walletBlocked(_that.reason);case _:
  return null;

}
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




/// @nodoc


class WalletState_Registration extends WalletState {
  const WalletState_Registration(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Registration);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.registration()';
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
  

 final  WalletTransferRole role;

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
 WalletTransferRole role
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
as WalletTransferRole,
  ));
}


}

/// @nodoc


class WalletState_Disclosure extends WalletState {
  const WalletState_Disclosure(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Disclosure);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.disclosure()';
}


}




/// @nodoc


class WalletState_Issuance extends WalletState {
  const WalletState_Issuance({required this.pid}): super._();
  

 final  bool pid;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WalletState_IssuanceCopyWith<WalletState_Issuance> get copyWith => _$WalletState_IssuanceCopyWithImpl<WalletState_Issuance>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_Issuance&&(identical(other.pid, pid) || other.pid == pid));
}


@override
int get hashCode => Object.hash(runtimeType,pid);

@override
String toString() {
  return 'WalletState.issuance(pid: $pid)';
}


}

/// @nodoc
abstract mixin class $WalletState_IssuanceCopyWith<$Res> implements $WalletStateCopyWith<$Res> {
  factory $WalletState_IssuanceCopyWith(WalletState_Issuance value, $Res Function(WalletState_Issuance) _then) = _$WalletState_IssuanceCopyWithImpl;
@useResult
$Res call({
 bool pid
});




}
/// @nodoc
class _$WalletState_IssuanceCopyWithImpl<$Res>
    implements $WalletState_IssuanceCopyWith<$Res> {
  _$WalletState_IssuanceCopyWithImpl(this._self, this._then);

  final WalletState_Issuance _self;
  final $Res Function(WalletState_Issuance) _then;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? pid = null,}) {
  return _then(WalletState_Issuance(
pid: null == pid ? _self.pid : pid // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class WalletState_PinChange extends WalletState {
  const WalletState_PinChange(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_PinChange);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.pinChange()';
}


}




/// @nodoc


class WalletState_PinRecovery extends WalletState {
  const WalletState_PinRecovery(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_PinRecovery);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WalletState.pinRecovery()';
}


}




/// @nodoc


class WalletState_WalletBlocked extends WalletState {
  const WalletState_WalletBlocked({required this.reason}): super._();
  

 final  WalletBlockedReason reason;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WalletState_WalletBlockedCopyWith<WalletState_WalletBlocked> get copyWith => _$WalletState_WalletBlockedCopyWithImpl<WalletState_WalletBlocked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WalletState_WalletBlocked&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'WalletState.walletBlocked(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $WalletState_WalletBlockedCopyWith<$Res> implements $WalletStateCopyWith<$Res> {
  factory $WalletState_WalletBlockedCopyWith(WalletState_WalletBlocked value, $Res Function(WalletState_WalletBlocked) _then) = _$WalletState_WalletBlockedCopyWithImpl;
@useResult
$Res call({
 WalletBlockedReason reason
});




}
/// @nodoc
class _$WalletState_WalletBlockedCopyWithImpl<$Res>
    implements $WalletState_WalletBlockedCopyWith<$Res> {
  _$WalletState_WalletBlockedCopyWithImpl(this._self, this._then);

  final WalletState_WalletBlocked _self;
  final $Res Function(WalletState_WalletBlocked) _then;

/// Create a copy of WalletState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(WalletState_WalletBlocked(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as WalletBlockedReason,
  ));
}


}

// dart format on
