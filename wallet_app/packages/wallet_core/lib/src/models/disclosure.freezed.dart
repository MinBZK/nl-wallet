// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'disclosure.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$AcceptDisclosureResult {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String? returnUrl) ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String? returnUrl)? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String? returnUrl)? ok,
    TResult Function(WalletInstructionError error)? instructionError,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AcceptDisclosureResult_Ok value) ok,
    required TResult Function(AcceptDisclosureResult_InstructionError value)
        instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AcceptDisclosureResult_Ok value)? ok,
    TResult? Function(AcceptDisclosureResult_InstructionError value)?
        instructionError,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AcceptDisclosureResult_Ok value)? ok,
    TResult Function(AcceptDisclosureResult_InstructionError value)?
        instructionError,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AcceptDisclosureResultCopyWith<$Res> {
  factory $AcceptDisclosureResultCopyWith(AcceptDisclosureResult value,
          $Res Function(AcceptDisclosureResult) then) =
      _$AcceptDisclosureResultCopyWithImpl<$Res, AcceptDisclosureResult>;
}

/// @nodoc
class _$AcceptDisclosureResultCopyWithImpl<$Res,
        $Val extends AcceptDisclosureResult>
    implements $AcceptDisclosureResultCopyWith<$Res> {
  _$AcceptDisclosureResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$AcceptDisclosureResult_OkImplCopyWith<$Res> {
  factory _$$AcceptDisclosureResult_OkImplCopyWith(
          _$AcceptDisclosureResult_OkImpl value,
          $Res Function(_$AcceptDisclosureResult_OkImpl) then) =
      __$$AcceptDisclosureResult_OkImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String? returnUrl});
}

/// @nodoc
class __$$AcceptDisclosureResult_OkImplCopyWithImpl<$Res>
    extends _$AcceptDisclosureResultCopyWithImpl<$Res,
        _$AcceptDisclosureResult_OkImpl>
    implements _$$AcceptDisclosureResult_OkImplCopyWith<$Res> {
  __$$AcceptDisclosureResult_OkImplCopyWithImpl(
      _$AcceptDisclosureResult_OkImpl _value,
      $Res Function(_$AcceptDisclosureResult_OkImpl) _then)
      : super(_value, _then);

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? returnUrl = freezed,
  }) {
    return _then(_$AcceptDisclosureResult_OkImpl(
      returnUrl: freezed == returnUrl
          ? _value.returnUrl
          : returnUrl // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc

class _$AcceptDisclosureResult_OkImpl extends AcceptDisclosureResult_Ok {
  const _$AcceptDisclosureResult_OkImpl({this.returnUrl}) : super._();

  @override
  final String? returnUrl;

  @override
  String toString() {
    return 'AcceptDisclosureResult.ok(returnUrl: $returnUrl)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AcceptDisclosureResult_OkImpl &&
            (identical(other.returnUrl, returnUrl) ||
                other.returnUrl == returnUrl));
  }

  @override
  int get hashCode => Object.hash(runtimeType, returnUrl);

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AcceptDisclosureResult_OkImplCopyWith<_$AcceptDisclosureResult_OkImpl>
      get copyWith => __$$AcceptDisclosureResult_OkImplCopyWithImpl<
          _$AcceptDisclosureResult_OkImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String? returnUrl) ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) {
    return ok(returnUrl);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String? returnUrl)? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) {
    return ok?.call(returnUrl);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String? returnUrl)? ok,
    TResult Function(WalletInstructionError error)? instructionError,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok(returnUrl);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AcceptDisclosureResult_Ok value) ok,
    required TResult Function(AcceptDisclosureResult_InstructionError value)
        instructionError,
  }) {
    return ok(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AcceptDisclosureResult_Ok value)? ok,
    TResult? Function(AcceptDisclosureResult_InstructionError value)?
        instructionError,
  }) {
    return ok?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AcceptDisclosureResult_Ok value)? ok,
    TResult Function(AcceptDisclosureResult_InstructionError value)?
        instructionError,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok(this);
    }
    return orElse();
  }
}

abstract class AcceptDisclosureResult_Ok extends AcceptDisclosureResult {
  const factory AcceptDisclosureResult_Ok({final String? returnUrl}) =
      _$AcceptDisclosureResult_OkImpl;
  const AcceptDisclosureResult_Ok._() : super._();

  String? get returnUrl;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AcceptDisclosureResult_OkImplCopyWith<_$AcceptDisclosureResult_OkImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$AcceptDisclosureResult_InstructionErrorImplCopyWith<$Res> {
  factory _$$AcceptDisclosureResult_InstructionErrorImplCopyWith(
          _$AcceptDisclosureResult_InstructionErrorImpl value,
          $Res Function(_$AcceptDisclosureResult_InstructionErrorImpl) then) =
      __$$AcceptDisclosureResult_InstructionErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({WalletInstructionError error});

  $WalletInstructionErrorCopyWith<$Res> get error;
}

/// @nodoc
class __$$AcceptDisclosureResult_InstructionErrorImplCopyWithImpl<$Res>
    extends _$AcceptDisclosureResultCopyWithImpl<$Res,
        _$AcceptDisclosureResult_InstructionErrorImpl>
    implements _$$AcceptDisclosureResult_InstructionErrorImplCopyWith<$Res> {
  __$$AcceptDisclosureResult_InstructionErrorImplCopyWithImpl(
      _$AcceptDisclosureResult_InstructionErrorImpl _value,
      $Res Function(_$AcceptDisclosureResult_InstructionErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? error = null,
  }) {
    return _then(_$AcceptDisclosureResult_InstructionErrorImpl(
      error: null == error
          ? _value.error
          : error // ignore: cast_nullable_to_non_nullable
              as WalletInstructionError,
    ));
  }

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $WalletInstructionErrorCopyWith<$Res> get error {
    return $WalletInstructionErrorCopyWith<$Res>(_value.error, (value) {
      return _then(_value.copyWith(error: value));
    });
  }
}

/// @nodoc

class _$AcceptDisclosureResult_InstructionErrorImpl
    extends AcceptDisclosureResult_InstructionError {
  const _$AcceptDisclosureResult_InstructionErrorImpl({required this.error})
      : super._();

  @override
  final WalletInstructionError error;

  @override
  String toString() {
    return 'AcceptDisclosureResult.instructionError(error: $error)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AcceptDisclosureResult_InstructionErrorImpl &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AcceptDisclosureResult_InstructionErrorImplCopyWith<
          _$AcceptDisclosureResult_InstructionErrorImpl>
      get copyWith =>
          __$$AcceptDisclosureResult_InstructionErrorImplCopyWithImpl<
              _$AcceptDisclosureResult_InstructionErrorImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String? returnUrl) ok,
    required TResult Function(WalletInstructionError error) instructionError,
  }) {
    return instructionError(error);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String? returnUrl)? ok,
    TResult? Function(WalletInstructionError error)? instructionError,
  }) {
    return instructionError?.call(error);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String? returnUrl)? ok,
    TResult Function(WalletInstructionError error)? instructionError,
    required TResult orElse(),
  }) {
    if (instructionError != null) {
      return instructionError(error);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AcceptDisclosureResult_Ok value) ok,
    required TResult Function(AcceptDisclosureResult_InstructionError value)
        instructionError,
  }) {
    return instructionError(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AcceptDisclosureResult_Ok value)? ok,
    TResult? Function(AcceptDisclosureResult_InstructionError value)?
        instructionError,
  }) {
    return instructionError?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AcceptDisclosureResult_Ok value)? ok,
    TResult Function(AcceptDisclosureResult_InstructionError value)?
        instructionError,
    required TResult orElse(),
  }) {
    if (instructionError != null) {
      return instructionError(this);
    }
    return orElse();
  }
}

abstract class AcceptDisclosureResult_InstructionError
    extends AcceptDisclosureResult {
  const factory AcceptDisclosureResult_InstructionError(
          {required final WalletInstructionError error}) =
      _$AcceptDisclosureResult_InstructionErrorImpl;
  const AcceptDisclosureResult_InstructionError._() : super._();

  WalletInstructionError get error;

  /// Create a copy of AcceptDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AcceptDisclosureResult_InstructionErrorImplCopyWith<
          _$AcceptDisclosureResult_InstructionErrorImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$Image {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ImageCopyWith<$Res> {
  factory $ImageCopyWith(Image value, $Res Function(Image) then) =
      _$ImageCopyWithImpl<$Res, Image>;
}

/// @nodoc
class _$ImageCopyWithImpl<$Res, $Val extends Image>
    implements $ImageCopyWith<$Res> {
  _$ImageCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$Image_SvgImplCopyWith<$Res> {
  factory _$$Image_SvgImplCopyWith(
          _$Image_SvgImpl value, $Res Function(_$Image_SvgImpl) then) =
      __$$Image_SvgImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String xml});
}

/// @nodoc
class __$$Image_SvgImplCopyWithImpl<$Res>
    extends _$ImageCopyWithImpl<$Res, _$Image_SvgImpl>
    implements _$$Image_SvgImplCopyWith<$Res> {
  __$$Image_SvgImplCopyWithImpl(
      _$Image_SvgImpl _value, $Res Function(_$Image_SvgImpl) _then)
      : super(_value, _then);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? xml = null,
  }) {
    return _then(_$Image_SvgImpl(
      xml: null == xml
          ? _value.xml
          : xml // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$Image_SvgImpl extends Image_Svg {
  const _$Image_SvgImpl({required this.xml}) : super._();

  @override
  final String xml;

  @override
  String toString() {
    return 'Image.svg(xml: $xml)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Image_SvgImpl &&
            (identical(other.xml, xml) || other.xml == xml));
  }

  @override
  int get hashCode => Object.hash(runtimeType, xml);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Image_SvgImplCopyWith<_$Image_SvgImpl> get copyWith =>
      __$$Image_SvgImplCopyWithImpl<_$Image_SvgImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) {
    return svg(xml);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) {
    return svg?.call(xml);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    if (svg != null) {
      return svg(xml);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) {
    return svg(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    return svg?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    if (svg != null) {
      return svg(this);
    }
    return orElse();
  }
}

abstract class Image_Svg extends Image {
  const factory Image_Svg({required final String xml}) = _$Image_SvgImpl;
  const Image_Svg._() : super._();

  String get xml;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Image_SvgImplCopyWith<_$Image_SvgImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Image_PngImplCopyWith<$Res> {
  factory _$$Image_PngImplCopyWith(
          _$Image_PngImpl value, $Res Function(_$Image_PngImpl) then) =
      __$$Image_PngImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String base64});
}

/// @nodoc
class __$$Image_PngImplCopyWithImpl<$Res>
    extends _$ImageCopyWithImpl<$Res, _$Image_PngImpl>
    implements _$$Image_PngImplCopyWith<$Res> {
  __$$Image_PngImplCopyWithImpl(
      _$Image_PngImpl _value, $Res Function(_$Image_PngImpl) _then)
      : super(_value, _then);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? base64 = null,
  }) {
    return _then(_$Image_PngImpl(
      base64: null == base64
          ? _value.base64
          : base64 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$Image_PngImpl extends Image_Png {
  const _$Image_PngImpl({required this.base64}) : super._();

  @override
  final String base64;

  @override
  String toString() {
    return 'Image.png(base64: $base64)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Image_PngImpl &&
            (identical(other.base64, base64) || other.base64 == base64));
  }

  @override
  int get hashCode => Object.hash(runtimeType, base64);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Image_PngImplCopyWith<_$Image_PngImpl> get copyWith =>
      __$$Image_PngImplCopyWithImpl<_$Image_PngImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) {
    return png(base64);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) {
    return png?.call(base64);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    if (png != null) {
      return png(base64);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) {
    return png(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    return png?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    if (png != null) {
      return png(this);
    }
    return orElse();
  }
}

abstract class Image_Png extends Image {
  const factory Image_Png({required final String base64}) = _$Image_PngImpl;
  const Image_Png._() : super._();

  String get base64;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Image_PngImplCopyWith<_$Image_PngImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Image_JpgImplCopyWith<$Res> {
  factory _$$Image_JpgImplCopyWith(
          _$Image_JpgImpl value, $Res Function(_$Image_JpgImpl) then) =
      __$$Image_JpgImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String base64});
}

/// @nodoc
class __$$Image_JpgImplCopyWithImpl<$Res>
    extends _$ImageCopyWithImpl<$Res, _$Image_JpgImpl>
    implements _$$Image_JpgImplCopyWith<$Res> {
  __$$Image_JpgImplCopyWithImpl(
      _$Image_JpgImpl _value, $Res Function(_$Image_JpgImpl) _then)
      : super(_value, _then);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? base64 = null,
  }) {
    return _then(_$Image_JpgImpl(
      base64: null == base64
          ? _value.base64
          : base64 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$Image_JpgImpl extends Image_Jpg {
  const _$Image_JpgImpl({required this.base64}) : super._();

  @override
  final String base64;

  @override
  String toString() {
    return 'Image.jpg(base64: $base64)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Image_JpgImpl &&
            (identical(other.base64, base64) || other.base64 == base64));
  }

  @override
  int get hashCode => Object.hash(runtimeType, base64);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Image_JpgImplCopyWith<_$Image_JpgImpl> get copyWith =>
      __$$Image_JpgImplCopyWithImpl<_$Image_JpgImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) {
    return jpg(base64);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) {
    return jpg?.call(base64);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    if (jpg != null) {
      return jpg(base64);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) {
    return jpg(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    return jpg?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    if (jpg != null) {
      return jpg(this);
    }
    return orElse();
  }
}

abstract class Image_Jpg extends Image {
  const factory Image_Jpg({required final String base64}) = _$Image_JpgImpl;
  const Image_Jpg._() : super._();

  String get base64;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Image_JpgImplCopyWith<_$Image_JpgImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Image_AssetImplCopyWith<$Res> {
  factory _$$Image_AssetImplCopyWith(
          _$Image_AssetImpl value, $Res Function(_$Image_AssetImpl) then) =
      __$$Image_AssetImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String path});
}

/// @nodoc
class __$$Image_AssetImplCopyWithImpl<$Res>
    extends _$ImageCopyWithImpl<$Res, _$Image_AssetImpl>
    implements _$$Image_AssetImplCopyWith<$Res> {
  __$$Image_AssetImplCopyWithImpl(
      _$Image_AssetImpl _value, $Res Function(_$Image_AssetImpl) _then)
      : super(_value, _then);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? path = null,
  }) {
    return _then(_$Image_AssetImpl(
      path: null == path
          ? _value.path
          : path // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$Image_AssetImpl extends Image_Asset {
  const _$Image_AssetImpl({required this.path}) : super._();

  @override
  final String path;

  @override
  String toString() {
    return 'Image.asset(path: $path)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Image_AssetImpl &&
            (identical(other.path, path) || other.path == path));
  }

  @override
  int get hashCode => Object.hash(runtimeType, path);

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Image_AssetImplCopyWith<_$Image_AssetImpl> get copyWith =>
      __$$Image_AssetImplCopyWithImpl<_$Image_AssetImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(String base64) png,
    required TResult Function(String base64) jpg,
    required TResult Function(String path) asset,
  }) {
    return asset(path);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(String base64)? png,
    TResult? Function(String base64)? jpg,
    TResult? Function(String path)? asset,
  }) {
    return asset?.call(path);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(String base64)? png,
    TResult Function(String base64)? jpg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    if (asset != null) {
      return asset(path);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpg value) jpg,
    required TResult Function(Image_Asset value) asset,
  }) {
    return asset(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpg value)? jpg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    return asset?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpg value)? jpg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    if (asset != null) {
      return asset(this);
    }
    return orElse();
  }
}

abstract class Image_Asset extends Image {
  const factory Image_Asset({required final String path}) = _$Image_AssetImpl;
  const Image_Asset._() : super._();

  String get path;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Image_AssetImplCopyWith<_$Image_AssetImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$StartDisclosureResult {
  Organization get relyingParty => throw _privateConstructorUsedError;
  bool get sharedDataWithRelyingPartyBefore =>
      throw _privateConstructorUsedError;
  DisclosureSessionType get sessionType => throw _privateConstructorUsedError;
  List<LocalizedString> get requestPurpose =>
      throw _privateConstructorUsedError;
  String get requestOriginBaseUrl => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)
        request,
    required TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)
        requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult? Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(StartDisclosureResult_Request value) request,
    required TResult Function(
            StartDisclosureResult_RequestAttributesMissing value)
        requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StartDisclosureResult_Request value)? request,
    TResult? Function(StartDisclosureResult_RequestAttributesMissing value)?
        requestAttributesMissing,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StartDisclosureResult_Request value)? request,
    TResult Function(StartDisclosureResult_RequestAttributesMissing value)?
        requestAttributesMissing,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $StartDisclosureResultCopyWith<StartDisclosureResult> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $StartDisclosureResultCopyWith<$Res> {
  factory $StartDisclosureResultCopyWith(StartDisclosureResult value,
          $Res Function(StartDisclosureResult) then) =
      _$StartDisclosureResultCopyWithImpl<$Res, StartDisclosureResult>;
  @useResult
  $Res call(
      {Organization relyingParty,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl});
}

/// @nodoc
class _$StartDisclosureResultCopyWithImpl<$Res,
        $Val extends StartDisclosureResult>
    implements $StartDisclosureResultCopyWith<$Res> {
  _$StartDisclosureResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
  }) {
    return _then(_value.copyWith(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _value.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _value.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _value.requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _value.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$StartDisclosureResult_RequestImplCopyWith<$Res>
    implements $StartDisclosureResultCopyWith<$Res> {
  factory _$$StartDisclosureResult_RequestImplCopyWith(
          _$StartDisclosureResult_RequestImpl value,
          $Res Function(_$StartDisclosureResult_RequestImpl) then) =
      __$$StartDisclosureResult_RequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {Organization relyingParty,
      RequestPolicy policy,
      List<Attestation> requestedAttestations,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl,
      DisclosureType requestType});
}

/// @nodoc
class __$$StartDisclosureResult_RequestImplCopyWithImpl<$Res>
    extends _$StartDisclosureResultCopyWithImpl<$Res,
        _$StartDisclosureResult_RequestImpl>
    implements _$$StartDisclosureResult_RequestImplCopyWith<$Res> {
  __$$StartDisclosureResult_RequestImplCopyWithImpl(
      _$StartDisclosureResult_RequestImpl _value,
      $Res Function(_$StartDisclosureResult_RequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? policy = null,
    Object? requestedAttestations = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
    Object? requestType = null,
  }) {
    return _then(_$StartDisclosureResult_RequestImpl(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      policy: null == policy
          ? _value.policy
          : policy // ignore: cast_nullable_to_non_nullable
              as RequestPolicy,
      requestedAttestations: null == requestedAttestations
          ? _value._requestedAttestations
          : requestedAttestations // ignore: cast_nullable_to_non_nullable
              as List<Attestation>,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _value.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _value.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _value._requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _value.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
      requestType: null == requestType
          ? _value.requestType
          : requestType // ignore: cast_nullable_to_non_nullable
              as DisclosureType,
    ));
  }
}

/// @nodoc

class _$StartDisclosureResult_RequestImpl
    extends StartDisclosureResult_Request {
  const _$StartDisclosureResult_RequestImpl(
      {required this.relyingParty,
      required this.policy,
      required final List<Attestation> requestedAttestations,
      required this.sharedDataWithRelyingPartyBefore,
      required this.sessionType,
      required final List<LocalizedString> requestPurpose,
      required this.requestOriginBaseUrl,
      required this.requestType})
      : _requestedAttestations = requestedAttestations,
        _requestPurpose = requestPurpose,
        super._();

  @override
  final Organization relyingParty;
  @override
  final RequestPolicy policy;
  final List<Attestation> _requestedAttestations;
  @override
  List<Attestation> get requestedAttestations {
    if (_requestedAttestations is EqualUnmodifiableListView)
      return _requestedAttestations;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestedAttestations);
  }

  @override
  final bool sharedDataWithRelyingPartyBefore;
  @override
  final DisclosureSessionType sessionType;
  final List<LocalizedString> _requestPurpose;
  @override
  List<LocalizedString> get requestPurpose {
    if (_requestPurpose is EqualUnmodifiableListView) return _requestPurpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestPurpose);
  }

  @override
  final String requestOriginBaseUrl;
  @override
  final DisclosureType requestType;

  @override
  String toString() {
    return 'StartDisclosureResult.request(relyingParty: $relyingParty, policy: $policy, requestedAttestations: $requestedAttestations, sharedDataWithRelyingPartyBefore: $sharedDataWithRelyingPartyBefore, sessionType: $sessionType, requestPurpose: $requestPurpose, requestOriginBaseUrl: $requestOriginBaseUrl, requestType: $requestType)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StartDisclosureResult_RequestImpl &&
            (identical(other.relyingParty, relyingParty) ||
                other.relyingParty == relyingParty) &&
            (identical(other.policy, policy) || other.policy == policy) &&
            const DeepCollectionEquality()
                .equals(other._requestedAttestations, _requestedAttestations) &&
            (identical(other.sharedDataWithRelyingPartyBefore,
                    sharedDataWithRelyingPartyBefore) ||
                other.sharedDataWithRelyingPartyBefore ==
                    sharedDataWithRelyingPartyBefore) &&
            (identical(other.sessionType, sessionType) ||
                other.sessionType == sessionType) &&
            const DeepCollectionEquality()
                .equals(other._requestPurpose, _requestPurpose) &&
            (identical(other.requestOriginBaseUrl, requestOriginBaseUrl) ||
                other.requestOriginBaseUrl == requestOriginBaseUrl) &&
            (identical(other.requestType, requestType) ||
                other.requestType == requestType));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      relyingParty,
      policy,
      const DeepCollectionEquality().hash(_requestedAttestations),
      sharedDataWithRelyingPartyBefore,
      sessionType,
      const DeepCollectionEquality().hash(_requestPurpose),
      requestOriginBaseUrl,
      requestType);

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$StartDisclosureResult_RequestImplCopyWith<
          _$StartDisclosureResult_RequestImpl>
      get copyWith => __$$StartDisclosureResult_RequestImplCopyWithImpl<
          _$StartDisclosureResult_RequestImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)
        request,
    required TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)
        requestAttributesMissing,
  }) {
    return request(
        relyingParty,
        policy,
        requestedAttestations,
        sharedDataWithRelyingPartyBefore,
        sessionType,
        requestPurpose,
        requestOriginBaseUrl,
        requestType);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult? Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
  }) {
    return request?.call(
        relyingParty,
        policy,
        requestedAttestations,
        sharedDataWithRelyingPartyBefore,
        sessionType,
        requestPurpose,
        requestOriginBaseUrl,
        requestType);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (request != null) {
      return request(
          relyingParty,
          policy,
          requestedAttestations,
          sharedDataWithRelyingPartyBefore,
          sessionType,
          requestPurpose,
          requestOriginBaseUrl,
          requestType);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(StartDisclosureResult_Request value) request,
    required TResult Function(
            StartDisclosureResult_RequestAttributesMissing value)
        requestAttributesMissing,
  }) {
    return request(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StartDisclosureResult_Request value)? request,
    TResult? Function(StartDisclosureResult_RequestAttributesMissing value)?
        requestAttributesMissing,
  }) {
    return request?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StartDisclosureResult_Request value)? request,
    TResult Function(StartDisclosureResult_RequestAttributesMissing value)?
        requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (request != null) {
      return request(this);
    }
    return orElse();
  }
}

abstract class StartDisclosureResult_Request extends StartDisclosureResult {
  const factory StartDisclosureResult_Request(
          {required final Organization relyingParty,
          required final RequestPolicy policy,
          required final List<Attestation> requestedAttestations,
          required final bool sharedDataWithRelyingPartyBefore,
          required final DisclosureSessionType sessionType,
          required final List<LocalizedString> requestPurpose,
          required final String requestOriginBaseUrl,
          required final DisclosureType requestType}) =
      _$StartDisclosureResult_RequestImpl;
  const StartDisclosureResult_Request._() : super._();

  @override
  Organization get relyingParty;
  RequestPolicy get policy;
  List<Attestation> get requestedAttestations;
  @override
  bool get sharedDataWithRelyingPartyBefore;
  @override
  DisclosureSessionType get sessionType;
  @override
  List<LocalizedString> get requestPurpose;
  @override
  String get requestOriginBaseUrl;
  DisclosureType get requestType;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$StartDisclosureResult_RequestImplCopyWith<
          _$StartDisclosureResult_RequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<
    $Res> implements $StartDisclosureResultCopyWith<$Res> {
  factory _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith(
          _$StartDisclosureResult_RequestAttributesMissingImpl value,
          $Res Function(_$StartDisclosureResult_RequestAttributesMissingImpl)
              then) =
      __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {Organization relyingParty,
      List<MissingAttribute> missingAttributes,
      bool sharedDataWithRelyingPartyBefore,
      DisclosureSessionType sessionType,
      List<LocalizedString> requestPurpose,
      String requestOriginBaseUrl});
}

/// @nodoc
class __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl<$Res>
    extends _$StartDisclosureResultCopyWithImpl<$Res,
        _$StartDisclosureResult_RequestAttributesMissingImpl>
    implements
        _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<$Res> {
  __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl(
      _$StartDisclosureResult_RequestAttributesMissingImpl _value,
      $Res Function(_$StartDisclosureResult_RequestAttributesMissingImpl) _then)
      : super(_value, _then);

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? relyingParty = null,
    Object? missingAttributes = null,
    Object? sharedDataWithRelyingPartyBefore = null,
    Object? sessionType = null,
    Object? requestPurpose = null,
    Object? requestOriginBaseUrl = null,
  }) {
    return _then(_$StartDisclosureResult_RequestAttributesMissingImpl(
      relyingParty: null == relyingParty
          ? _value.relyingParty
          : relyingParty // ignore: cast_nullable_to_non_nullable
              as Organization,
      missingAttributes: null == missingAttributes
          ? _value._missingAttributes
          : missingAttributes // ignore: cast_nullable_to_non_nullable
              as List<MissingAttribute>,
      sharedDataWithRelyingPartyBefore: null == sharedDataWithRelyingPartyBefore
          ? _value.sharedDataWithRelyingPartyBefore
          : sharedDataWithRelyingPartyBefore // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionType: null == sessionType
          ? _value.sessionType
          : sessionType // ignore: cast_nullable_to_non_nullable
              as DisclosureSessionType,
      requestPurpose: null == requestPurpose
          ? _value._requestPurpose
          : requestPurpose // ignore: cast_nullable_to_non_nullable
              as List<LocalizedString>,
      requestOriginBaseUrl: null == requestOriginBaseUrl
          ? _value.requestOriginBaseUrl
          : requestOriginBaseUrl // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$StartDisclosureResult_RequestAttributesMissingImpl
    extends StartDisclosureResult_RequestAttributesMissing {
  const _$StartDisclosureResult_RequestAttributesMissingImpl(
      {required this.relyingParty,
      required final List<MissingAttribute> missingAttributes,
      required this.sharedDataWithRelyingPartyBefore,
      required this.sessionType,
      required final List<LocalizedString> requestPurpose,
      required this.requestOriginBaseUrl})
      : _missingAttributes = missingAttributes,
        _requestPurpose = requestPurpose,
        super._();

  @override
  final Organization relyingParty;
  final List<MissingAttribute> _missingAttributes;
  @override
  List<MissingAttribute> get missingAttributes {
    if (_missingAttributes is EqualUnmodifiableListView)
      return _missingAttributes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_missingAttributes);
  }

  @override
  final bool sharedDataWithRelyingPartyBefore;
  @override
  final DisclosureSessionType sessionType;
  final List<LocalizedString> _requestPurpose;
  @override
  List<LocalizedString> get requestPurpose {
    if (_requestPurpose is EqualUnmodifiableListView) return _requestPurpose;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_requestPurpose);
  }

  @override
  final String requestOriginBaseUrl;

  @override
  String toString() {
    return 'StartDisclosureResult.requestAttributesMissing(relyingParty: $relyingParty, missingAttributes: $missingAttributes, sharedDataWithRelyingPartyBefore: $sharedDataWithRelyingPartyBefore, sessionType: $sessionType, requestPurpose: $requestPurpose, requestOriginBaseUrl: $requestOriginBaseUrl)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StartDisclosureResult_RequestAttributesMissingImpl &&
            (identical(other.relyingParty, relyingParty) ||
                other.relyingParty == relyingParty) &&
            const DeepCollectionEquality()
                .equals(other._missingAttributes, _missingAttributes) &&
            (identical(other.sharedDataWithRelyingPartyBefore,
                    sharedDataWithRelyingPartyBefore) ||
                other.sharedDataWithRelyingPartyBefore ==
                    sharedDataWithRelyingPartyBefore) &&
            (identical(other.sessionType, sessionType) ||
                other.sessionType == sessionType) &&
            const DeepCollectionEquality()
                .equals(other._requestPurpose, _requestPurpose) &&
            (identical(other.requestOriginBaseUrl, requestOriginBaseUrl) ||
                other.requestOriginBaseUrl == requestOriginBaseUrl));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      relyingParty,
      const DeepCollectionEquality().hash(_missingAttributes),
      sharedDataWithRelyingPartyBefore,
      sessionType,
      const DeepCollectionEquality().hash(_requestPurpose),
      requestOriginBaseUrl);

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<
          _$StartDisclosureResult_RequestAttributesMissingImpl>
      get copyWith =>
          __$$StartDisclosureResult_RequestAttributesMissingImplCopyWithImpl<
                  _$StartDisclosureResult_RequestAttributesMissingImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)
        request,
    required TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)
        requestAttributesMissing,
  }) {
    return requestAttributesMissing(
        relyingParty,
        missingAttributes,
        sharedDataWithRelyingPartyBefore,
        sessionType,
        requestPurpose,
        requestOriginBaseUrl);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult? Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
  }) {
    return requestAttributesMissing?.call(
        relyingParty,
        missingAttributes,
        sharedDataWithRelyingPartyBefore,
        sessionType,
        requestPurpose,
        requestOriginBaseUrl);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            Organization relyingParty,
            RequestPolicy policy,
            List<Attestation> requestedAttestations,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl,
            DisclosureType requestType)?
        request,
    TResult Function(
            Organization relyingParty,
            List<MissingAttribute> missingAttributes,
            bool sharedDataWithRelyingPartyBefore,
            DisclosureSessionType sessionType,
            List<LocalizedString> requestPurpose,
            String requestOriginBaseUrl)?
        requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (requestAttributesMissing != null) {
      return requestAttributesMissing(
          relyingParty,
          missingAttributes,
          sharedDataWithRelyingPartyBefore,
          sessionType,
          requestPurpose,
          requestOriginBaseUrl);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(StartDisclosureResult_Request value) request,
    required TResult Function(
            StartDisclosureResult_RequestAttributesMissing value)
        requestAttributesMissing,
  }) {
    return requestAttributesMissing(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StartDisclosureResult_Request value)? request,
    TResult? Function(StartDisclosureResult_RequestAttributesMissing value)?
        requestAttributesMissing,
  }) {
    return requestAttributesMissing?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StartDisclosureResult_Request value)? request,
    TResult Function(StartDisclosureResult_RequestAttributesMissing value)?
        requestAttributesMissing,
    required TResult orElse(),
  }) {
    if (requestAttributesMissing != null) {
      return requestAttributesMissing(this);
    }
    return orElse();
  }
}

abstract class StartDisclosureResult_RequestAttributesMissing
    extends StartDisclosureResult {
  const factory StartDisclosureResult_RequestAttributesMissing(
          {required final Organization relyingParty,
          required final List<MissingAttribute> missingAttributes,
          required final bool sharedDataWithRelyingPartyBefore,
          required final DisclosureSessionType sessionType,
          required final List<LocalizedString> requestPurpose,
          required final String requestOriginBaseUrl}) =
      _$StartDisclosureResult_RequestAttributesMissingImpl;
  const StartDisclosureResult_RequestAttributesMissing._() : super._();

  @override
  Organization get relyingParty;
  List<MissingAttribute> get missingAttributes;
  @override
  bool get sharedDataWithRelyingPartyBefore;
  @override
  DisclosureSessionType get sessionType;
  @override
  List<LocalizedString> get requestPurpose;
  @override
  String get requestOriginBaseUrl;

  /// Create a copy of StartDisclosureResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$StartDisclosureResult_RequestAttributesMissingImplCopyWith<
          _$StartDisclosureResult_RequestAttributesMissingImpl>
      get copyWith => throw _privateConstructorUsedError;
}
