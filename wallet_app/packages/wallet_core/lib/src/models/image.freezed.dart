// dart format width=80
// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'image.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Image {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is Image);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'Image()';
  }
}

/// @nodoc
class $ImageCopyWith<$Res> {
  $ImageCopyWith(Image _, $Res Function(Image) __);
}

/// @nodoc

class Image_Svg extends Image {
  const Image_Svg({required this.xml}) : super._();

  final String xml;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Image_SvgCopyWith<Image_Svg> get copyWith => _$Image_SvgCopyWithImpl<Image_Svg>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is Image_Svg && (identical(other.xml, xml) || other.xml == xml));
  }

  @override
  int get hashCode => Object.hash(runtimeType, xml);

  @override
  String toString() {
    return 'Image.svg(xml: $xml)';
  }
}

/// @nodoc
abstract mixin class $Image_SvgCopyWith<$Res> implements $ImageCopyWith<$Res> {
  factory $Image_SvgCopyWith(Image_Svg value, $Res Function(Image_Svg) _then) = _$Image_SvgCopyWithImpl;
  @useResult
  $Res call({String xml});
}

/// @nodoc
class _$Image_SvgCopyWithImpl<$Res> implements $Image_SvgCopyWith<$Res> {
  _$Image_SvgCopyWithImpl(this._self, this._then);

  final Image_Svg _self;
  final $Res Function(Image_Svg) _then;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? xml = null,
  }) {
    return _then(Image_Svg(
      xml: null == xml
          ? _self.xml
          : xml // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class Image_Png extends Image {
  const Image_Png({required this.data}) : super._();

  final Uint8List data;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Image_PngCopyWith<Image_Png> get copyWith => _$Image_PngCopyWithImpl<Image_Png>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Image_Png &&
            const DeepCollectionEquality().equals(other.data, data));
  }

  @override
  int get hashCode => Object.hash(runtimeType, const DeepCollectionEquality().hash(data));

  @override
  String toString() {
    return 'Image.png(data: $data)';
  }
}

/// @nodoc
abstract mixin class $Image_PngCopyWith<$Res> implements $ImageCopyWith<$Res> {
  factory $Image_PngCopyWith(Image_Png value, $Res Function(Image_Png) _then) = _$Image_PngCopyWithImpl;
  @useResult
  $Res call({Uint8List data});
}

/// @nodoc
class _$Image_PngCopyWithImpl<$Res> implements $Image_PngCopyWith<$Res> {
  _$Image_PngCopyWithImpl(this._self, this._then);

  final Image_Png _self;
  final $Res Function(Image_Png) _then;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? data = null,
  }) {
    return _then(Image_Png(
      data: null == data
          ? _self.data
          : data // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// @nodoc

class Image_Jpeg extends Image {
  const Image_Jpeg({required this.data}) : super._();

  final Uint8List data;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Image_JpegCopyWith<Image_Jpeg> get copyWith => _$Image_JpegCopyWithImpl<Image_Jpeg>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Image_Jpeg &&
            const DeepCollectionEquality().equals(other.data, data));
  }

  @override
  int get hashCode => Object.hash(runtimeType, const DeepCollectionEquality().hash(data));

  @override
  String toString() {
    return 'Image.jpeg(data: $data)';
  }
}

/// @nodoc
abstract mixin class $Image_JpegCopyWith<$Res> implements $ImageCopyWith<$Res> {
  factory $Image_JpegCopyWith(Image_Jpeg value, $Res Function(Image_Jpeg) _then) = _$Image_JpegCopyWithImpl;
  @useResult
  $Res call({Uint8List data});
}

/// @nodoc
class _$Image_JpegCopyWithImpl<$Res> implements $Image_JpegCopyWith<$Res> {
  _$Image_JpegCopyWithImpl(this._self, this._then);

  final Image_Jpeg _self;
  final $Res Function(Image_Jpeg) _then;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? data = null,
  }) {
    return _then(Image_Jpeg(
      data: null == data
          ? _self.data
          : data // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// @nodoc

class Image_Asset extends Image {
  const Image_Asset({required this.path}) : super._();

  final String path;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Image_AssetCopyWith<Image_Asset> get copyWith => _$Image_AssetCopyWithImpl<Image_Asset>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Image_Asset &&
            (identical(other.path, path) || other.path == path));
  }

  @override
  int get hashCode => Object.hash(runtimeType, path);

  @override
  String toString() {
    return 'Image.asset(path: $path)';
  }
}

/// @nodoc
abstract mixin class $Image_AssetCopyWith<$Res> implements $ImageCopyWith<$Res> {
  factory $Image_AssetCopyWith(Image_Asset value, $Res Function(Image_Asset) _then) = _$Image_AssetCopyWithImpl;
  @useResult
  $Res call({String path});
}

/// @nodoc
class _$Image_AssetCopyWithImpl<$Res> implements $Image_AssetCopyWith<$Res> {
  _$Image_AssetCopyWithImpl(this._self, this._then);

  final Image_Asset _self;
  final $Res Function(Image_Asset) _then;

  /// Create a copy of Image
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? path = null,
  }) {
    return _then(Image_Asset(
      path: null == path
          ? _self.path
          : path // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

// dart format on
