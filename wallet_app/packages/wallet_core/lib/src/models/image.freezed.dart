// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
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

/// Adds pattern-matching-related methods to [Image].
extension ImagePatterns on Image {
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

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Image_Svg value)? svg,
    TResult Function(Image_Png value)? png,
    TResult Function(Image_Jpeg value)? jpeg,
    TResult Function(Image_Asset value)? asset,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case Image_Svg() when svg != null:
        return svg(_that);
      case Image_Png() when png != null:
        return png(_that);
      case Image_Jpeg() when jpeg != null:
        return jpeg(_that);
      case Image_Asset() when asset != null:
        return asset(_that);
      case _:
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

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Image_Svg value) svg,
    required TResult Function(Image_Png value) png,
    required TResult Function(Image_Jpeg value) jpeg,
    required TResult Function(Image_Asset value) asset,
  }) {
    final _that = this;
    switch (_that) {
      case Image_Svg():
        return svg(_that);
      case Image_Png():
        return png(_that);
      case Image_Jpeg():
        return jpeg(_that);
      case Image_Asset():
        return asset(_that);
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

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Image_Svg value)? svg,
    TResult? Function(Image_Png value)? png,
    TResult? Function(Image_Jpeg value)? jpeg,
    TResult? Function(Image_Asset value)? asset,
  }) {
    final _that = this;
    switch (_that) {
      case Image_Svg() when svg != null:
        return svg(_that);
      case Image_Png() when png != null:
        return png(_that);
      case Image_Jpeg() when jpeg != null:
        return jpeg(_that);
      case Image_Asset() when asset != null:
        return asset(_that);
      case _:
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

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String xml)? svg,
    TResult Function(Uint8List data)? png,
    TResult Function(Uint8List data)? jpeg,
    TResult Function(String path)? asset,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case Image_Svg() when svg != null:
        return svg(_that.xml);
      case Image_Png() when png != null:
        return png(_that.data);
      case Image_Jpeg() when jpeg != null:
        return jpeg(_that.data);
      case Image_Asset() when asset != null:
        return asset(_that.path);
      case _:
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

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String xml) svg,
    required TResult Function(Uint8List data) png,
    required TResult Function(Uint8List data) jpeg,
    required TResult Function(String path) asset,
  }) {
    final _that = this;
    switch (_that) {
      case Image_Svg():
        return svg(_that.xml);
      case Image_Png():
        return png(_that.data);
      case Image_Jpeg():
        return jpeg(_that.data);
      case Image_Asset():
        return asset(_that.path);
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

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String xml)? svg,
    TResult? Function(Uint8List data)? png,
    TResult? Function(Uint8List data)? jpeg,
    TResult? Function(String path)? asset,
  }) {
    final _that = this;
    switch (_that) {
      case Image_Svg() when svg != null:
        return svg(_that.xml);
      case Image_Png() when png != null:
        return png(_that.data);
      case Image_Jpeg() when jpeg != null:
        return jpeg(_that.data);
      case Image_Asset() when asset != null:
        return asset(_that.path);
      case _:
        return null;
    }
  }
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
