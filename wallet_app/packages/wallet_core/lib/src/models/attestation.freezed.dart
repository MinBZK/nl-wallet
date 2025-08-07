// dart format width=80
// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'attestation.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$AttestationIdentity {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is AttestationIdentity);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'AttestationIdentity()';
  }
}

/// @nodoc
class $AttestationIdentityCopyWith<$Res> {
  $AttestationIdentityCopyWith(AttestationIdentity _, $Res Function(AttestationIdentity) __);
}

/// @nodoc

class AttestationIdentity_Ephemeral extends AttestationIdentity {
  const AttestationIdentity_Ephemeral() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is AttestationIdentity_Ephemeral);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'AttestationIdentity.ephemeral()';
  }
}

/// @nodoc

class AttestationIdentity_Fixed extends AttestationIdentity {
  const AttestationIdentity_Fixed({required this.id}) : super._();

  final String id;

  /// Create a copy of AttestationIdentity
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AttestationIdentity_FixedCopyWith<AttestationIdentity_Fixed> get copyWith =>
      _$AttestationIdentity_FixedCopyWithImpl<AttestationIdentity_Fixed>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AttestationIdentity_Fixed &&
            (identical(other.id, id) || other.id == id));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id);

  @override
  String toString() {
    return 'AttestationIdentity.fixed(id: $id)';
  }
}

/// @nodoc
abstract mixin class $AttestationIdentity_FixedCopyWith<$Res> implements $AttestationIdentityCopyWith<$Res> {
  factory $AttestationIdentity_FixedCopyWith(
          AttestationIdentity_Fixed value, $Res Function(AttestationIdentity_Fixed) _then) =
      _$AttestationIdentity_FixedCopyWithImpl;
  @useResult
  $Res call({String id});
}

/// @nodoc
class _$AttestationIdentity_FixedCopyWithImpl<$Res> implements $AttestationIdentity_FixedCopyWith<$Res> {
  _$AttestationIdentity_FixedCopyWithImpl(this._self, this._then);

  final AttestationIdentity_Fixed _self;
  final $Res Function(AttestationIdentity_Fixed) _then;

  /// Create a copy of AttestationIdentity
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
  }) {
    return _then(AttestationIdentity_Fixed(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$AttributeValue {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is AttributeValue);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'AttributeValue()';
  }
}

/// @nodoc
class $AttributeValueCopyWith<$Res> {
  $AttributeValueCopyWith(AttributeValue _, $Res Function(AttributeValue) __);
}

/// @nodoc

class AttributeValue_String extends AttributeValue {
  const AttributeValue_String({required this.value}) : super._();

  final String value;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AttributeValue_StringCopyWith<AttributeValue_String> get copyWith =>
      _$AttributeValue_StringCopyWithImpl<AttributeValue_String>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AttributeValue_String &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @override
  String toString() {
    return 'AttributeValue.string(value: $value)';
  }
}

/// @nodoc
abstract mixin class $AttributeValue_StringCopyWith<$Res> implements $AttributeValueCopyWith<$Res> {
  factory $AttributeValue_StringCopyWith(AttributeValue_String value, $Res Function(AttributeValue_String) _then) =
      _$AttributeValue_StringCopyWithImpl;
  @useResult
  $Res call({String value});
}

/// @nodoc
class _$AttributeValue_StringCopyWithImpl<$Res> implements $AttributeValue_StringCopyWith<$Res> {
  _$AttributeValue_StringCopyWithImpl(this._self, this._then);

  final AttributeValue_String _self;
  final $Res Function(AttributeValue_String) _then;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? value = null,
  }) {
    return _then(AttributeValue_String(
      value: null == value
          ? _self.value
          : value // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class AttributeValue_Boolean extends AttributeValue {
  const AttributeValue_Boolean({required this.value}) : super._();

  final bool value;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AttributeValue_BooleanCopyWith<AttributeValue_Boolean> get copyWith =>
      _$AttributeValue_BooleanCopyWithImpl<AttributeValue_Boolean>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AttributeValue_Boolean &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @override
  String toString() {
    return 'AttributeValue.boolean(value: $value)';
  }
}

/// @nodoc
abstract mixin class $AttributeValue_BooleanCopyWith<$Res> implements $AttributeValueCopyWith<$Res> {
  factory $AttributeValue_BooleanCopyWith(AttributeValue_Boolean value, $Res Function(AttributeValue_Boolean) _then) =
      _$AttributeValue_BooleanCopyWithImpl;
  @useResult
  $Res call({bool value});
}

/// @nodoc
class _$AttributeValue_BooleanCopyWithImpl<$Res> implements $AttributeValue_BooleanCopyWith<$Res> {
  _$AttributeValue_BooleanCopyWithImpl(this._self, this._then);

  final AttributeValue_Boolean _self;
  final $Res Function(AttributeValue_Boolean) _then;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? value = null,
  }) {
    return _then(AttributeValue_Boolean(
      value: null == value
          ? _self.value
          : value // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc

class AttributeValue_Number extends AttributeValue {
  const AttributeValue_Number({required this.value}) : super._();

  final int value;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AttributeValue_NumberCopyWith<AttributeValue_Number> get copyWith =>
      _$AttributeValue_NumberCopyWithImpl<AttributeValue_Number>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AttributeValue_Number &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @override
  String toString() {
    return 'AttributeValue.number(value: $value)';
  }
}

/// @nodoc
abstract mixin class $AttributeValue_NumberCopyWith<$Res> implements $AttributeValueCopyWith<$Res> {
  factory $AttributeValue_NumberCopyWith(AttributeValue_Number value, $Res Function(AttributeValue_Number) _then) =
      _$AttributeValue_NumberCopyWithImpl;
  @useResult
  $Res call({int value});
}

/// @nodoc
class _$AttributeValue_NumberCopyWithImpl<$Res> implements $AttributeValue_NumberCopyWith<$Res> {
  _$AttributeValue_NumberCopyWithImpl(this._self, this._then);

  final AttributeValue_Number _self;
  final $Res Function(AttributeValue_Number) _then;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? value = null,
  }) {
    return _then(AttributeValue_Number(
      value: null == value
          ? _self.value
          : value // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class AttributeValue_Date extends AttributeValue {
  const AttributeValue_Date({required this.value}) : super._();

  final String value;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AttributeValue_DateCopyWith<AttributeValue_Date> get copyWith =>
      _$AttributeValue_DateCopyWithImpl<AttributeValue_Date>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AttributeValue_Date &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  @override
  String toString() {
    return 'AttributeValue.date(value: $value)';
  }
}

/// @nodoc
abstract mixin class $AttributeValue_DateCopyWith<$Res> implements $AttributeValueCopyWith<$Res> {
  factory $AttributeValue_DateCopyWith(AttributeValue_Date value, $Res Function(AttributeValue_Date) _then) =
      _$AttributeValue_DateCopyWithImpl;
  @useResult
  $Res call({String value});
}

/// @nodoc
class _$AttributeValue_DateCopyWithImpl<$Res> implements $AttributeValue_DateCopyWith<$Res> {
  _$AttributeValue_DateCopyWithImpl(this._self, this._then);

  final AttributeValue_Date _self;
  final $Res Function(AttributeValue_Date) _then;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? value = null,
  }) {
    return _then(AttributeValue_Date(
      value: null == value
          ? _self.value
          : value // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class AttributeValue_Array extends AttributeValue {
  const AttributeValue_Array({required final List<AttributeValue> value})
      : _value = value,
        super._();

  final List<AttributeValue> _value;
  List<AttributeValue> get value {
    if (_value is EqualUnmodifiableListView) return _value;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_value);
  }

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AttributeValue_ArrayCopyWith<AttributeValue_Array> get copyWith =>
      _$AttributeValue_ArrayCopyWithImpl<AttributeValue_Array>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AttributeValue_Array &&
            const DeepCollectionEquality().equals(other._value, _value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, const DeepCollectionEquality().hash(_value));

  @override
  String toString() {
    return 'AttributeValue.array(value: $value)';
  }
}

/// @nodoc
abstract mixin class $AttributeValue_ArrayCopyWith<$Res> implements $AttributeValueCopyWith<$Res> {
  factory $AttributeValue_ArrayCopyWith(AttributeValue_Array value, $Res Function(AttributeValue_Array) _then) =
      _$AttributeValue_ArrayCopyWithImpl;
  @useResult
  $Res call({List<AttributeValue> value});
}

/// @nodoc
class _$AttributeValue_ArrayCopyWithImpl<$Res> implements $AttributeValue_ArrayCopyWith<$Res> {
  _$AttributeValue_ArrayCopyWithImpl(this._self, this._then);

  final AttributeValue_Array _self;
  final $Res Function(AttributeValue_Array) _then;

  /// Create a copy of AttributeValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? value = null,
  }) {
    return _then(AttributeValue_Array(
      value: null == value
          ? _self._value
          : value // ignore: cast_nullable_to_non_nullable
              as List<AttributeValue>,
    ));
  }
}

/// @nodoc

class AttributeValue_Null extends AttributeValue {
  const AttributeValue_Null() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is AttributeValue_Null);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'AttributeValue.null_()';
  }
}

/// @nodoc
mixin _$RenderingMetadata {
  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is RenderingMetadata);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'RenderingMetadata()';
  }
}

/// @nodoc
class $RenderingMetadataCopyWith<$Res> {
  $RenderingMetadataCopyWith(RenderingMetadata _, $Res Function(RenderingMetadata) __);
}

/// @nodoc

class RenderingMetadata_Simple extends RenderingMetadata {
  const RenderingMetadata_Simple({this.logo, this.backgroundColor, this.textColor}) : super._();

  final ImageWithMetadata? logo;
  final String? backgroundColor;
  final String? textColor;

  /// Create a copy of RenderingMetadata
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $RenderingMetadata_SimpleCopyWith<RenderingMetadata_Simple> get copyWith =>
      _$RenderingMetadata_SimpleCopyWithImpl<RenderingMetadata_Simple>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is RenderingMetadata_Simple &&
            (identical(other.logo, logo) || other.logo == logo) &&
            (identical(other.backgroundColor, backgroundColor) || other.backgroundColor == backgroundColor) &&
            (identical(other.textColor, textColor) || other.textColor == textColor));
  }

  @override
  int get hashCode => Object.hash(runtimeType, logo, backgroundColor, textColor);

  @override
  String toString() {
    return 'RenderingMetadata.simple(logo: $logo, backgroundColor: $backgroundColor, textColor: $textColor)';
  }
}

/// @nodoc
abstract mixin class $RenderingMetadata_SimpleCopyWith<$Res> implements $RenderingMetadataCopyWith<$Res> {
  factory $RenderingMetadata_SimpleCopyWith(
          RenderingMetadata_Simple value, $Res Function(RenderingMetadata_Simple) _then) =
      _$RenderingMetadata_SimpleCopyWithImpl;
  @useResult
  $Res call({ImageWithMetadata? logo, String? backgroundColor, String? textColor});
}

/// @nodoc
class _$RenderingMetadata_SimpleCopyWithImpl<$Res> implements $RenderingMetadata_SimpleCopyWith<$Res> {
  _$RenderingMetadata_SimpleCopyWithImpl(this._self, this._then);

  final RenderingMetadata_Simple _self;
  final $Res Function(RenderingMetadata_Simple) _then;

  /// Create a copy of RenderingMetadata
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? logo = freezed,
    Object? backgroundColor = freezed,
    Object? textColor = freezed,
  }) {
    return _then(RenderingMetadata_Simple(
      logo: freezed == logo
          ? _self.logo
          : logo // ignore: cast_nullable_to_non_nullable
              as ImageWithMetadata?,
      backgroundColor: freezed == backgroundColor
          ? _self.backgroundColor
          : backgroundColor // ignore: cast_nullable_to_non_nullable
              as String?,
      textColor: freezed == textColor
          ? _self.textColor
          : textColor // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc

class RenderingMetadata_SvgTemplates extends RenderingMetadata {
  const RenderingMetadata_SvgTemplates() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType && other is RenderingMetadata_SvgTemplates);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'RenderingMetadata.svgTemplates()';
  }
}

// dart format on
