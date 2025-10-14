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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WalletState_Ready value)?  ready,TResult Function( WalletState_Transferring value)?  transferring,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WalletState_Ready() when ready != null:
return ready(_that);case WalletState_Transferring() when transferring != null:
return transferring(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WalletState_Ready value)  ready,required TResult Function( WalletState_Transferring value)  transferring,}){
final _that = this;
switch (_that) {
case WalletState_Ready():
return ready(_that);case WalletState_Transferring():
return transferring(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WalletState_Ready value)?  ready,TResult? Function( WalletState_Transferring value)?  transferring,}){
final _that = this;
switch (_that) {
case WalletState_Ready() when ready != null:
return ready(_that);case WalletState_Transferring() when transferring != null:
return transferring(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  ready,TResult Function( WalletTransferRole role)?  transferring,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WalletState_Ready() when ready != null:
return ready();case WalletState_Transferring() when transferring != null:
return transferring(_that.role);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  ready,required TResult Function( WalletTransferRole role)  transferring,}) {final _that = this;
switch (_that) {
case WalletState_Ready():
return ready();case WalletState_Transferring():
return transferring(_that.role);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  ready,TResult? Function( WalletTransferRole role)?  transferring,}) {final _that = this;
switch (_that) {
case WalletState_Ready() when ready != null:
return ready();case WalletState_Transferring() when transferring != null:
return transferring(_that.role);case _:
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

// dart format on
