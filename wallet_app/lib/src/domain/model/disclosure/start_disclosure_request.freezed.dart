// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'start_disclosure_request.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
StartDisclosureRequest _$StartDisclosureRequestFromJson(
  Map<String, dynamic> json
) {
        switch (json['runtimeType']) {
                  case 'deeplink':
          return DeeplinkStartDisclosureRequest.fromJson(
            json
          );
                case 'qrScan':
          return QrScanStartDisclosureRequest.fromJson(
            json
          );
                case 'closeProximity':
          return CloseProximityStartDisclosureRequest.fromJson(
            json
          );
        
          default:
            throw CheckedFromJsonException(
  json,
  'runtimeType',
  'StartDisclosureRequest',
  'Invalid union type "${json['runtimeType']}"!'
);
        }
      
}

/// @nodoc
mixin _$StartDisclosureRequest {



  /// Serializes this StartDisclosureRequest to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StartDisclosureRequest);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'StartDisclosureRequest()';
}


}

/// @nodoc
class $StartDisclosureRequestCopyWith<$Res>  {
$StartDisclosureRequestCopyWith(StartDisclosureRequest _, $Res Function(StartDisclosureRequest) __);
}


/// Adds pattern-matching-related methods to [StartDisclosureRequest].
extension StartDisclosureRequestPatterns on StartDisclosureRequest {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DeeplinkStartDisclosureRequest value)?  deeplink,TResult Function( QrScanStartDisclosureRequest value)?  qrScan,TResult Function( CloseProximityStartDisclosureRequest value)?  closeProximity,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DeeplinkStartDisclosureRequest() when deeplink != null:
return deeplink(_that);case QrScanStartDisclosureRequest() when qrScan != null:
return qrScan(_that);case CloseProximityStartDisclosureRequest() when closeProximity != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DeeplinkStartDisclosureRequest value)  deeplink,required TResult Function( QrScanStartDisclosureRequest value)  qrScan,required TResult Function( CloseProximityStartDisclosureRequest value)  closeProximity,}){
final _that = this;
switch (_that) {
case DeeplinkStartDisclosureRequest():
return deeplink(_that);case QrScanStartDisclosureRequest():
return qrScan(_that);case CloseProximityStartDisclosureRequest():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DeeplinkStartDisclosureRequest value)?  deeplink,TResult? Function( QrScanStartDisclosureRequest value)?  qrScan,TResult? Function( CloseProximityStartDisclosureRequest value)?  closeProximity,}){
final _that = this;
switch (_that) {
case DeeplinkStartDisclosureRequest() when deeplink != null:
return deeplink(_that);case QrScanStartDisclosureRequest() when qrScan != null:
return qrScan(_that);case CloseProximityStartDisclosureRequest() when closeProximity != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String uri)?  deeplink,TResult Function( String uri)?  qrScan,TResult Function()?  closeProximity,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DeeplinkStartDisclosureRequest() when deeplink != null:
return deeplink(_that.uri);case QrScanStartDisclosureRequest() when qrScan != null:
return qrScan(_that.uri);case CloseProximityStartDisclosureRequest() when closeProximity != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String uri)  deeplink,required TResult Function( String uri)  qrScan,required TResult Function()  closeProximity,}) {final _that = this;
switch (_that) {
case DeeplinkStartDisclosureRequest():
return deeplink(_that.uri);case QrScanStartDisclosureRequest():
return qrScan(_that.uri);case CloseProximityStartDisclosureRequest():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String uri)?  deeplink,TResult? Function( String uri)?  qrScan,TResult? Function()?  closeProximity,}) {final _that = this;
switch (_that) {
case DeeplinkStartDisclosureRequest() when deeplink != null:
return deeplink(_that.uri);case QrScanStartDisclosureRequest() when qrScan != null:
return qrScan(_that.uri);case CloseProximityStartDisclosureRequest() when closeProximity != null:
return closeProximity();case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class DeeplinkStartDisclosureRequest implements StartDisclosureRequest {
  const DeeplinkStartDisclosureRequest(this.uri, {final  String? $type}): $type = $type ?? 'deeplink';
  factory DeeplinkStartDisclosureRequest.fromJson(Map<String, dynamic> json) => _$DeeplinkStartDisclosureRequestFromJson(json);

 final  String uri;

@JsonKey(name: 'runtimeType')
final String $type;


/// Create a copy of StartDisclosureRequest
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeeplinkStartDisclosureRequestCopyWith<DeeplinkStartDisclosureRequest> get copyWith => _$DeeplinkStartDisclosureRequestCopyWithImpl<DeeplinkStartDisclosureRequest>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DeeplinkStartDisclosureRequestToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeeplinkStartDisclosureRequest&&(identical(other.uri, uri) || other.uri == uri));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,uri);

@override
String toString() {
  return 'StartDisclosureRequest.deeplink(uri: $uri)';
}


}

/// @nodoc
abstract mixin class $DeeplinkStartDisclosureRequestCopyWith<$Res> implements $StartDisclosureRequestCopyWith<$Res> {
  factory $DeeplinkStartDisclosureRequestCopyWith(DeeplinkStartDisclosureRequest value, $Res Function(DeeplinkStartDisclosureRequest) _then) = _$DeeplinkStartDisclosureRequestCopyWithImpl;
@useResult
$Res call({
 String uri
});




}
/// @nodoc
class _$DeeplinkStartDisclosureRequestCopyWithImpl<$Res>
    implements $DeeplinkStartDisclosureRequestCopyWith<$Res> {
  _$DeeplinkStartDisclosureRequestCopyWithImpl(this._self, this._then);

  final DeeplinkStartDisclosureRequest _self;
  final $Res Function(DeeplinkStartDisclosureRequest) _then;

/// Create a copy of StartDisclosureRequest
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? uri = null,}) {
  return _then(DeeplinkStartDisclosureRequest(
null == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
@JsonSerializable()

class QrScanStartDisclosureRequest implements StartDisclosureRequest {
  const QrScanStartDisclosureRequest(this.uri, {final  String? $type}): $type = $type ?? 'qrScan';
  factory QrScanStartDisclosureRequest.fromJson(Map<String, dynamic> json) => _$QrScanStartDisclosureRequestFromJson(json);

 final  String uri;

@JsonKey(name: 'runtimeType')
final String $type;


/// Create a copy of StartDisclosureRequest
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QrScanStartDisclosureRequestCopyWith<QrScanStartDisclosureRequest> get copyWith => _$QrScanStartDisclosureRequestCopyWithImpl<QrScanStartDisclosureRequest>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$QrScanStartDisclosureRequestToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QrScanStartDisclosureRequest&&(identical(other.uri, uri) || other.uri == uri));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,uri);

@override
String toString() {
  return 'StartDisclosureRequest.qrScan(uri: $uri)';
}


}

/// @nodoc
abstract mixin class $QrScanStartDisclosureRequestCopyWith<$Res> implements $StartDisclosureRequestCopyWith<$Res> {
  factory $QrScanStartDisclosureRequestCopyWith(QrScanStartDisclosureRequest value, $Res Function(QrScanStartDisclosureRequest) _then) = _$QrScanStartDisclosureRequestCopyWithImpl;
@useResult
$Res call({
 String uri
});




}
/// @nodoc
class _$QrScanStartDisclosureRequestCopyWithImpl<$Res>
    implements $QrScanStartDisclosureRequestCopyWith<$Res> {
  _$QrScanStartDisclosureRequestCopyWithImpl(this._self, this._then);

  final QrScanStartDisclosureRequest _self;
  final $Res Function(QrScanStartDisclosureRequest) _then;

/// Create a copy of StartDisclosureRequest
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? uri = null,}) {
  return _then(QrScanStartDisclosureRequest(
null == uri ? _self.uri : uri // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
@JsonSerializable()

class CloseProximityStartDisclosureRequest implements StartDisclosureRequest {
  const CloseProximityStartDisclosureRequest({final  String? $type}): $type = $type ?? 'closeProximity';
  factory CloseProximityStartDisclosureRequest.fromJson(Map<String, dynamic> json) => _$CloseProximityStartDisclosureRequestFromJson(json);



@JsonKey(name: 'runtimeType')
final String $type;



@override
Map<String, dynamic> toJson() {
  return _$CloseProximityStartDisclosureRequestToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CloseProximityStartDisclosureRequest);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'StartDisclosureRequest.closeProximity()';
}


}




// dart format on
