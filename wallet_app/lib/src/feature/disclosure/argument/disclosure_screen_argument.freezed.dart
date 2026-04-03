// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'disclosure_screen_argument.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$DisclosureScreenArgument {

 DisclosureConnectionType get type;
/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DisclosureScreenArgumentCopyWith<DisclosureScreenArgument> get copyWith => _$DisclosureScreenArgumentCopyWithImpl<DisclosureScreenArgument>(this as DisclosureScreenArgument, _$identity);

  /// Serializes this DisclosureScreenArgument to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DisclosureScreenArgument&&(identical(other.type, type) || other.type == type));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,type);

@override
String toString() {
  return 'DisclosureScreenArgument(type: $type)';
}


}

/// @nodoc
abstract mixin class $DisclosureScreenArgumentCopyWith<$Res>  {
  factory $DisclosureScreenArgumentCopyWith(DisclosureScreenArgument value, $Res Function(DisclosureScreenArgument) _then) = _$DisclosureScreenArgumentCopyWithImpl;
@useResult
$Res call({
 DisclosureConnectionType type
});


$DisclosureConnectionTypeCopyWith<$Res> get type;

}
/// @nodoc
class _$DisclosureScreenArgumentCopyWithImpl<$Res>
    implements $DisclosureScreenArgumentCopyWith<$Res> {
  _$DisclosureScreenArgumentCopyWithImpl(this._self, this._then);

  final DisclosureScreenArgument _self;
  final $Res Function(DisclosureScreenArgument) _then;

/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? type = null,}) {
  return _then(_self.copyWith(
type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as DisclosureConnectionType,
  ));
}
/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DisclosureConnectionTypeCopyWith<$Res> get type {
  
  return $DisclosureConnectionTypeCopyWith<$Res>(_self.type, (value) {
    return _then(_self.copyWith(type: value));
  });
}
}


/// Adds pattern-matching-related methods to [DisclosureScreenArgument].
extension DisclosureScreenArgumentPatterns on DisclosureScreenArgument {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DisclosureScreenArgument value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DisclosureScreenArgument() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DisclosureScreenArgument value)  $default,){
final _that = this;
switch (_that) {
case _DisclosureScreenArgument():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DisclosureScreenArgument value)?  $default,){
final _that = this;
switch (_that) {
case _DisclosureScreenArgument() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( DisclosureConnectionType type)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DisclosureScreenArgument() when $default != null:
return $default(_that.type);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( DisclosureConnectionType type)  $default,) {final _that = this;
switch (_that) {
case _DisclosureScreenArgument():
return $default(_that.type);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( DisclosureConnectionType type)?  $default,) {final _that = this;
switch (_that) {
case _DisclosureScreenArgument() when $default != null:
return $default(_that.type);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DisclosureScreenArgument implements DisclosureScreenArgument {
  const _DisclosureScreenArgument({required this.type});
  factory _DisclosureScreenArgument.fromJson(Map<String, dynamic> json) => _$DisclosureScreenArgumentFromJson(json);

@override final  DisclosureConnectionType type;

/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DisclosureScreenArgumentCopyWith<_DisclosureScreenArgument> get copyWith => __$DisclosureScreenArgumentCopyWithImpl<_DisclosureScreenArgument>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DisclosureScreenArgumentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DisclosureScreenArgument&&(identical(other.type, type) || other.type == type));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,type);

@override
String toString() {
  return 'DisclosureScreenArgument(type: $type)';
}


}

/// @nodoc
abstract mixin class _$DisclosureScreenArgumentCopyWith<$Res> implements $DisclosureScreenArgumentCopyWith<$Res> {
  factory _$DisclosureScreenArgumentCopyWith(_DisclosureScreenArgument value, $Res Function(_DisclosureScreenArgument) _then) = __$DisclosureScreenArgumentCopyWithImpl;
@override @useResult
$Res call({
 DisclosureConnectionType type
});


@override $DisclosureConnectionTypeCopyWith<$Res> get type;

}
/// @nodoc
class __$DisclosureScreenArgumentCopyWithImpl<$Res>
    implements _$DisclosureScreenArgumentCopyWith<$Res> {
  __$DisclosureScreenArgumentCopyWithImpl(this._self, this._then);

  final _DisclosureScreenArgument _self;
  final $Res Function(_DisclosureScreenArgument) _then;

/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? type = null,}) {
  return _then(_DisclosureScreenArgument(
type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as DisclosureConnectionType,
  ));
}

/// Create a copy of DisclosureScreenArgument
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DisclosureConnectionTypeCopyWith<$Res> get type {
  
  return $DisclosureConnectionTypeCopyWith<$Res>(_self.type, (value) {
    return _then(_self.copyWith(type: value));
  });
}
}

DisclosureConnectionType _$DisclosureConnectionTypeFromJson(
  Map<String, dynamic> json
) {
        switch (json['runtimeType']) {
                  case 'remote':
          return RemoteDisclosure.fromJson(
            json
          );
                case 'closeProximity':
          return CloseProximityDisclosure.fromJson(
            json
          );
        
          default:
            throw CheckedFromJsonException(
  json,
  'runtimeType',
  'DisclosureConnectionType',
  'Invalid union type "${json['runtimeType']}"!'
);
        }
      
}

/// @nodoc
mixin _$DisclosureConnectionType {



  /// Serializes this DisclosureConnectionType to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DisclosureConnectionType);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DisclosureConnectionType()';
}


}

/// @nodoc
class $DisclosureConnectionTypeCopyWith<$Res>  {
$DisclosureConnectionTypeCopyWith(DisclosureConnectionType _, $Res Function(DisclosureConnectionType) __);
}


/// Adds pattern-matching-related methods to [DisclosureConnectionType].
extension DisclosureConnectionTypePatterns on DisclosureConnectionType {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( RemoteDisclosure value)?  remote,TResult Function( CloseProximityDisclosure value)?  closeProximity,required TResult orElse(),}){
final _that = this;
switch (_that) {
case RemoteDisclosure() when remote != null:
return remote(_that);case CloseProximityDisclosure() when closeProximity != null:
return closeProximity(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( RemoteDisclosure value)  remote,required TResult Function( CloseProximityDisclosure value)  closeProximity,}){
final _that = this;
switch (_that) {
case RemoteDisclosure():
return remote(_that);case CloseProximityDisclosure():
return closeProximity(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( RemoteDisclosure value)?  remote,TResult? Function( CloseProximityDisclosure value)?  closeProximity,}){
final _that = this;
switch (_that) {
case RemoteDisclosure() when remote != null:
return remote(_that);case CloseProximityDisclosure() when closeProximity != null:
return closeProximity(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String uri,  bool isQrCode)?  remote,TResult Function()?  closeProximity,required TResult orElse(),}) {final _that = this;
switch (_that) {
case RemoteDisclosure() when remote != null:
return remote(_that.uri,_that.isQrCode);case CloseProximityDisclosure() when closeProximity != null:
return closeProximity();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String uri,  bool isQrCode)  remote,required TResult Function()  closeProximity,}) {final _that = this;
switch (_that) {
case RemoteDisclosure():
return remote(_that.uri,_that.isQrCode);case CloseProximityDisclosure():
return closeProximity();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String uri,  bool isQrCode)?  remote,TResult? Function()?  closeProximity,}) {final _that = this;
switch (_that) {
case RemoteDisclosure() when remote != null:
return remote(_that.uri,_that.isQrCode);case CloseProximityDisclosure() when closeProximity != null:
return closeProximity();case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class RemoteDisclosure implements DisclosureConnectionType {
  const RemoteDisclosure(this.uri, {required this.isQrCode, final  String? $type}): $type = $type ?? 'remote';
  factory RemoteDisclosure.fromJson(Map<String, dynamic> json) => _$RemoteDisclosureFromJson(json);

 final  String uri;
 final  bool isQrCode;

@JsonKey(name: 'runtimeType')
final String $type;


/// Create a copy of DisclosureConnectionType
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RemoteDisclosureCopyWith<RemoteDisclosure> get copyWith => _$RemoteDisclosureCopyWithImpl<RemoteDisclosure>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$RemoteDisclosureToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RemoteDisclosure&&(identical(other.uri, uri) || other.uri == uri)&&(identical(other.isQrCode, isQrCode) || other.isQrCode == isQrCode));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,uri,isQrCode);

@override
String toString() {
  return 'DisclosureConnectionType.remote(uri: $uri, isQrCode: $isQrCode)';
}


}

/// @nodoc
abstract mixin class $RemoteDisclosureCopyWith<$Res> implements $DisclosureConnectionTypeCopyWith<$Res> {
  factory $RemoteDisclosureCopyWith(RemoteDisclosure value, $Res Function(RemoteDisclosure) _then) = _$RemoteDisclosureCopyWithImpl;
@useResult
$Res call({
 String uri, bool isQrCode
});




}
/// @nodoc
class _$RemoteDisclosureCopyWithImpl<$Res>
    implements $RemoteDisclosureCopyWith<$Res> {
  _$RemoteDisclosureCopyWithImpl(this._self, this._then);

  final RemoteDisclosure _self;
  final $Res Function(RemoteDisclosure) _then;

/// Create a copy of DisclosureConnectionType
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? uri = null,Object? isQrCode = null,}) {
  return _then(RemoteDisclosure(
null == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String,isQrCode: null == isQrCode ? _self.isQrCode : isQrCode // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
@JsonSerializable()

class CloseProximityDisclosure implements DisclosureConnectionType {
  const CloseProximityDisclosure({final  String? $type}): $type = $type ?? 'closeProximity';
  factory CloseProximityDisclosure.fromJson(Map<String, dynamic> json) => _$CloseProximityDisclosureFromJson(json);



@JsonKey(name: 'runtimeType')
final String $type;



@override
Map<String, dynamic> toJson() {
  return _$CloseProximityDisclosureToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CloseProximityDisclosure);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DisclosureConnectionType.closeProximity()';
}


}




// dart format on
