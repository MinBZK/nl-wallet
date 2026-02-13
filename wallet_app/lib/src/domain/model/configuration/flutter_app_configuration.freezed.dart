// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'flutter_app_configuration.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FlutterAppConfiguration {

 Duration get idleLockTimeout; Duration get idleWarningTimeout; Duration get backgroundLockTimeout; String get staticAssetsBaseUrl; List<String> get pidAttestationTypes; MaintenanceWindow? get maintenanceWindow; String get version; String get environment;
/// Create a copy of FlutterAppConfiguration
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FlutterAppConfigurationCopyWith<FlutterAppConfiguration> get copyWith => _$FlutterAppConfigurationCopyWithImpl<FlutterAppConfiguration>(this as FlutterAppConfiguration, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FlutterAppConfiguration&&(identical(other.idleLockTimeout, idleLockTimeout) || other.idleLockTimeout == idleLockTimeout)&&(identical(other.idleWarningTimeout, idleWarningTimeout) || other.idleWarningTimeout == idleWarningTimeout)&&(identical(other.backgroundLockTimeout, backgroundLockTimeout) || other.backgroundLockTimeout == backgroundLockTimeout)&&(identical(other.staticAssetsBaseUrl, staticAssetsBaseUrl) || other.staticAssetsBaseUrl == staticAssetsBaseUrl)&&const DeepCollectionEquality().equals(other.pidAttestationTypes, pidAttestationTypes)&&(identical(other.maintenanceWindow, maintenanceWindow) || other.maintenanceWindow == maintenanceWindow)&&(identical(other.version, version) || other.version == version)&&(identical(other.environment, environment) || other.environment == environment));
}


@override
int get hashCode => Object.hash(runtimeType,idleLockTimeout,idleWarningTimeout,backgroundLockTimeout,staticAssetsBaseUrl,const DeepCollectionEquality().hash(pidAttestationTypes),maintenanceWindow,version,environment);

@override
String toString() {
  return 'FlutterAppConfiguration(idleLockTimeout: $idleLockTimeout, idleWarningTimeout: $idleWarningTimeout, backgroundLockTimeout: $backgroundLockTimeout, staticAssetsBaseUrl: $staticAssetsBaseUrl, pidAttestationTypes: $pidAttestationTypes, maintenanceWindow: $maintenanceWindow, version: $version, environment: $environment)';
}


}

/// @nodoc
abstract mixin class $FlutterAppConfigurationCopyWith<$Res>  {
  factory $FlutterAppConfigurationCopyWith(FlutterAppConfiguration value, $Res Function(FlutterAppConfiguration) _then) = _$FlutterAppConfigurationCopyWithImpl;
@useResult
$Res call({
 Duration idleLockTimeout, Duration idleWarningTimeout, Duration backgroundLockTimeout, String staticAssetsBaseUrl, List<String> pidAttestationTypes, MaintenanceWindow? maintenanceWindow, String version, String environment
});


$MaintenanceWindowCopyWith<$Res>? get maintenanceWindow;

}
/// @nodoc
class _$FlutterAppConfigurationCopyWithImpl<$Res>
    implements $FlutterAppConfigurationCopyWith<$Res> {
  _$FlutterAppConfigurationCopyWithImpl(this._self, this._then);

  final FlutterAppConfiguration _self;
  final $Res Function(FlutterAppConfiguration) _then;

/// Create a copy of FlutterAppConfiguration
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? idleLockTimeout = null,Object? idleWarningTimeout = null,Object? backgroundLockTimeout = null,Object? staticAssetsBaseUrl = null,Object? pidAttestationTypes = null,Object? maintenanceWindow = freezed,Object? version = null,Object? environment = null,}) {
  return _then(_self.copyWith(
idleLockTimeout: null == idleLockTimeout ? _self.idleLockTimeout : idleLockTimeout // ignore: cast_nullable_to_non_nullable
as Duration,idleWarningTimeout: null == idleWarningTimeout ? _self.idleWarningTimeout : idleWarningTimeout // ignore: cast_nullable_to_non_nullable
as Duration,backgroundLockTimeout: null == backgroundLockTimeout ? _self.backgroundLockTimeout : backgroundLockTimeout // ignore: cast_nullable_to_non_nullable
as Duration,staticAssetsBaseUrl: null == staticAssetsBaseUrl ? _self.staticAssetsBaseUrl : staticAssetsBaseUrl // ignore: cast_nullable_to_non_nullable
as String,pidAttestationTypes: null == pidAttestationTypes ? _self.pidAttestationTypes : pidAttestationTypes // ignore: cast_nullable_to_non_nullable
as List<String>,maintenanceWindow: freezed == maintenanceWindow ? _self.maintenanceWindow : maintenanceWindow // ignore: cast_nullable_to_non_nullable
as MaintenanceWindow?,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String,environment: null == environment ? _self.environment : environment // ignore: cast_nullable_to_non_nullable
as String,
  ));
}
/// Create a copy of FlutterAppConfiguration
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$MaintenanceWindowCopyWith<$Res>? get maintenanceWindow {
    if (_self.maintenanceWindow == null) {
    return null;
  }

  return $MaintenanceWindowCopyWith<$Res>(_self.maintenanceWindow!, (value) {
    return _then(_self.copyWith(maintenanceWindow: value));
  });
}
}


/// Adds pattern-matching-related methods to [FlutterAppConfiguration].
extension FlutterAppConfigurationPatterns on FlutterAppConfiguration {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FlutterAppConfiguration value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FlutterAppConfiguration() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FlutterAppConfiguration value)  $default,){
final _that = this;
switch (_that) {
case _FlutterAppConfiguration():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FlutterAppConfiguration value)?  $default,){
final _that = this;
switch (_that) {
case _FlutterAppConfiguration() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( Duration idleLockTimeout,  Duration idleWarningTimeout,  Duration backgroundLockTimeout,  String staticAssetsBaseUrl,  List<String> pidAttestationTypes,  MaintenanceWindow? maintenanceWindow,  String version,  String environment)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FlutterAppConfiguration() when $default != null:
return $default(_that.idleLockTimeout,_that.idleWarningTimeout,_that.backgroundLockTimeout,_that.staticAssetsBaseUrl,_that.pidAttestationTypes,_that.maintenanceWindow,_that.version,_that.environment);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( Duration idleLockTimeout,  Duration idleWarningTimeout,  Duration backgroundLockTimeout,  String staticAssetsBaseUrl,  List<String> pidAttestationTypes,  MaintenanceWindow? maintenanceWindow,  String version,  String environment)  $default,) {final _that = this;
switch (_that) {
case _FlutterAppConfiguration():
return $default(_that.idleLockTimeout,_that.idleWarningTimeout,_that.backgroundLockTimeout,_that.staticAssetsBaseUrl,_that.pidAttestationTypes,_that.maintenanceWindow,_that.version,_that.environment);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( Duration idleLockTimeout,  Duration idleWarningTimeout,  Duration backgroundLockTimeout,  String staticAssetsBaseUrl,  List<String> pidAttestationTypes,  MaintenanceWindow? maintenanceWindow,  String version,  String environment)?  $default,) {final _that = this;
switch (_that) {
case _FlutterAppConfiguration() when $default != null:
return $default(_that.idleLockTimeout,_that.idleWarningTimeout,_that.backgroundLockTimeout,_that.staticAssetsBaseUrl,_that.pidAttestationTypes,_that.maintenanceWindow,_that.version,_that.environment);case _:
  return null;

}
}

}

/// @nodoc


class _FlutterAppConfiguration extends FlutterAppConfiguration {
  const _FlutterAppConfiguration({required this.idleLockTimeout, required this.idleWarningTimeout, required this.backgroundLockTimeout, required this.staticAssetsBaseUrl, required final  List<String> pidAttestationTypes, required this.maintenanceWindow, required this.version, required this.environment}): _pidAttestationTypes = pidAttestationTypes,super._();
  

@override final  Duration idleLockTimeout;
@override final  Duration idleWarningTimeout;
@override final  Duration backgroundLockTimeout;
@override final  String staticAssetsBaseUrl;
 final  List<String> _pidAttestationTypes;
@override List<String> get pidAttestationTypes {
  if (_pidAttestationTypes is EqualUnmodifiableListView) return _pidAttestationTypes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_pidAttestationTypes);
}

@override final  MaintenanceWindow? maintenanceWindow;
@override final  String version;
@override final  String environment;

/// Create a copy of FlutterAppConfiguration
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FlutterAppConfigurationCopyWith<_FlutterAppConfiguration> get copyWith => __$FlutterAppConfigurationCopyWithImpl<_FlutterAppConfiguration>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FlutterAppConfiguration&&(identical(other.idleLockTimeout, idleLockTimeout) || other.idleLockTimeout == idleLockTimeout)&&(identical(other.idleWarningTimeout, idleWarningTimeout) || other.idleWarningTimeout == idleWarningTimeout)&&(identical(other.backgroundLockTimeout, backgroundLockTimeout) || other.backgroundLockTimeout == backgroundLockTimeout)&&(identical(other.staticAssetsBaseUrl, staticAssetsBaseUrl) || other.staticAssetsBaseUrl == staticAssetsBaseUrl)&&const DeepCollectionEquality().equals(other._pidAttestationTypes, _pidAttestationTypes)&&(identical(other.maintenanceWindow, maintenanceWindow) || other.maintenanceWindow == maintenanceWindow)&&(identical(other.version, version) || other.version == version)&&(identical(other.environment, environment) || other.environment == environment));
}


@override
int get hashCode => Object.hash(runtimeType,idleLockTimeout,idleWarningTimeout,backgroundLockTimeout,staticAssetsBaseUrl,const DeepCollectionEquality().hash(_pidAttestationTypes),maintenanceWindow,version,environment);

@override
String toString() {
  return 'FlutterAppConfiguration(idleLockTimeout: $idleLockTimeout, idleWarningTimeout: $idleWarningTimeout, backgroundLockTimeout: $backgroundLockTimeout, staticAssetsBaseUrl: $staticAssetsBaseUrl, pidAttestationTypes: $pidAttestationTypes, maintenanceWindow: $maintenanceWindow, version: $version, environment: $environment)';
}


}

/// @nodoc
abstract mixin class _$FlutterAppConfigurationCopyWith<$Res> implements $FlutterAppConfigurationCopyWith<$Res> {
  factory _$FlutterAppConfigurationCopyWith(_FlutterAppConfiguration value, $Res Function(_FlutterAppConfiguration) _then) = __$FlutterAppConfigurationCopyWithImpl;
@override @useResult
$Res call({
 Duration idleLockTimeout, Duration idleWarningTimeout, Duration backgroundLockTimeout, String staticAssetsBaseUrl, List<String> pidAttestationTypes, MaintenanceWindow? maintenanceWindow, String version, String environment
});


@override $MaintenanceWindowCopyWith<$Res>? get maintenanceWindow;

}
/// @nodoc
class __$FlutterAppConfigurationCopyWithImpl<$Res>
    implements _$FlutterAppConfigurationCopyWith<$Res> {
  __$FlutterAppConfigurationCopyWithImpl(this._self, this._then);

  final _FlutterAppConfiguration _self;
  final $Res Function(_FlutterAppConfiguration) _then;

/// Create a copy of FlutterAppConfiguration
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? idleLockTimeout = null,Object? idleWarningTimeout = null,Object? backgroundLockTimeout = null,Object? staticAssetsBaseUrl = null,Object? pidAttestationTypes = null,Object? maintenanceWindow = freezed,Object? version = null,Object? environment = null,}) {
  return _then(_FlutterAppConfiguration(
idleLockTimeout: null == idleLockTimeout ? _self.idleLockTimeout : idleLockTimeout // ignore: cast_nullable_to_non_nullable
as Duration,idleWarningTimeout: null == idleWarningTimeout ? _self.idleWarningTimeout : idleWarningTimeout // ignore: cast_nullable_to_non_nullable
as Duration,backgroundLockTimeout: null == backgroundLockTimeout ? _self.backgroundLockTimeout : backgroundLockTimeout // ignore: cast_nullable_to_non_nullable
as Duration,staticAssetsBaseUrl: null == staticAssetsBaseUrl ? _self.staticAssetsBaseUrl : staticAssetsBaseUrl // ignore: cast_nullable_to_non_nullable
as String,pidAttestationTypes: null == pidAttestationTypes ? _self._pidAttestationTypes : pidAttestationTypes // ignore: cast_nullable_to_non_nullable
as List<String>,maintenanceWindow: freezed == maintenanceWindow ? _self.maintenanceWindow : maintenanceWindow // ignore: cast_nullable_to_non_nullable
as MaintenanceWindow?,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String,environment: null == environment ? _self.environment : environment // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

/// Create a copy of FlutterAppConfiguration
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$MaintenanceWindowCopyWith<$Res>? get maintenanceWindow {
    if (_self.maintenanceWindow == null) {
    return null;
  }

  return $MaintenanceWindowCopyWith<$Res>(_self.maintenanceWindow!, (value) {
    return _then(_self.copyWith(maintenanceWindow: value));
  });
}
}

// dart format on
