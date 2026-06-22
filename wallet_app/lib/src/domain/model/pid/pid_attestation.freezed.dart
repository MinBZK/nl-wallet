// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'pid_attestation.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PidAttestation {

 String get attestationType; AttestationFormat get format;
/// Create a copy of PidAttestation
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PidAttestationCopyWith<PidAttestation> get copyWith => _$PidAttestationCopyWithImpl<PidAttestation>(this as PidAttestation, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PidAttestation&&(identical(other.attestationType, attestationType) || other.attestationType == attestationType)&&(identical(other.format, format) || other.format == format));
}


@override
int get hashCode => Object.hash(runtimeType,attestationType,format);

@override
String toString() {
  return 'PidAttestation(attestationType: $attestationType, format: $format)';
}


}

/// @nodoc
abstract mixin class $PidAttestationCopyWith<$Res>  {
  factory $PidAttestationCopyWith(PidAttestation value, $Res Function(PidAttestation) _then) = _$PidAttestationCopyWithImpl;
@useResult
$Res call({
 String attestationType, AttestationFormat format
});




}
/// @nodoc
class _$PidAttestationCopyWithImpl<$Res>
    implements $PidAttestationCopyWith<$Res> {
  _$PidAttestationCopyWithImpl(this._self, this._then);

  final PidAttestation _self;
  final $Res Function(PidAttestation) _then;

/// Create a copy of PidAttestation
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? attestationType = null,Object? format = null,}) {
  return _then(_self.copyWith(
attestationType: null == attestationType ? _self.attestationType : attestationType // ignore: cast_nullable_to_non_nullable
as String,format: null == format ? _self.format : format // ignore: cast_nullable_to_non_nullable
as AttestationFormat,
  ));
}

}


/// Adds pattern-matching-related methods to [PidAttestation].
extension PidAttestationPatterns on PidAttestation {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PidAttestation value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PidAttestation() when $default != null:
return $default(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PidAttestation value)  $default,){
final _that = this;
switch (_that) {
case _PidAttestation():
return $default(_that);case _:
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PidAttestation value)?  $default,){
final _that = this;
switch (_that) {
case _PidAttestation() when $default != null:
return $default(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String attestationType,  AttestationFormat format)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PidAttestation() when $default != null:
return $default(_that.attestationType,_that.format);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String attestationType,  AttestationFormat format)  $default,) {final _that = this;
switch (_that) {
case _PidAttestation():
return $default(_that.attestationType,_that.format);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String attestationType,  AttestationFormat format)?  $default,) {final _that = this;
switch (_that) {
case _PidAttestation() when $default != null:
return $default(_that.attestationType,_that.format);case _:
  return null;

}
}

}

/// @nodoc


class _PidAttestation extends PidAttestation {
  const _PidAttestation({required this.attestationType, required this.format}): super._();
  

@override final  String attestationType;
@override final  AttestationFormat format;

/// Create a copy of PidAttestation
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PidAttestationCopyWith<_PidAttestation> get copyWith => __$PidAttestationCopyWithImpl<_PidAttestation>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PidAttestation&&(identical(other.attestationType, attestationType) || other.attestationType == attestationType)&&(identical(other.format, format) || other.format == format));
}


@override
int get hashCode => Object.hash(runtimeType,attestationType,format);

@override
String toString() {
  return 'PidAttestation(attestationType: $attestationType, format: $format)';
}


}

/// @nodoc
abstract mixin class _$PidAttestationCopyWith<$Res> implements $PidAttestationCopyWith<$Res> {
  factory _$PidAttestationCopyWith(_PidAttestation value, $Res Function(_PidAttestation) _then) = __$PidAttestationCopyWithImpl;
@override @useResult
$Res call({
 String attestationType, AttestationFormat format
});




}
/// @nodoc
class __$PidAttestationCopyWithImpl<$Res>
    implements _$PidAttestationCopyWith<$Res> {
  __$PidAttestationCopyWithImpl(this._self, this._then);

  final _PidAttestation _self;
  final $Res Function(_PidAttestation) _then;

/// Create a copy of PidAttestation
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? attestationType = null,Object? format = null,}) {
  return _then(_PidAttestation(
attestationType: null == attestationType ? _self.attestationType : attestationType // ignore: cast_nullable_to_non_nullable
as String,format: null == format ? _self.format : format // ignore: cast_nullable_to_non_nullable
as AttestationFormat,
  ));
}


}

// dart format on
