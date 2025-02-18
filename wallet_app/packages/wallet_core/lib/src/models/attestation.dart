// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.8.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'disclosure.dart';
import 'localize.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart' hide protected;
part 'attestation.freezed.dart';

class Attestation {
  final AttestationIdentity identity;
  final String attestationType;
  final List<DisplayMetadata> displayMetadata;
  final Organization issuer;
  final List<AttestationAttribute> attributes;

  const Attestation({
    required this.identity,
    required this.attestationType,
    required this.displayMetadata,
    required this.issuer,
    required this.attributes,
  });

  @override
  int get hashCode =>
      identity.hashCode ^ attestationType.hashCode ^ displayMetadata.hashCode ^ issuer.hashCode ^ attributes.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Attestation &&
          runtimeType == other.runtimeType &&
          identity == other.identity &&
          attestationType == other.attestationType &&
          displayMetadata == other.displayMetadata &&
          issuer == other.issuer &&
          attributes == other.attributes;
}

class AttestationAttribute {
  final String key;
  final List<LocalizedString> labels;
  final AttributeValue value;

  const AttestationAttribute({
    required this.key,
    required this.labels,
    required this.value,
  });

  @override
  int get hashCode => key.hashCode ^ labels.hashCode ^ value.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is AttestationAttribute &&
          runtimeType == other.runtimeType &&
          key == other.key &&
          labels == other.labels &&
          value == other.value;
}

@freezed
sealed class AttestationIdentity with _$AttestationIdentity {
  const AttestationIdentity._();

  const factory AttestationIdentity.ephemeral() = AttestationIdentity_Ephemeral;
  const factory AttestationIdentity.fixed({
    required String id,
  }) = AttestationIdentity_Fixed;
}

@freezed
sealed class AttributeValue with _$AttributeValue {
  const AttributeValue._();

  const factory AttributeValue.string({
    required String value,
  }) = AttributeValue_String;
  const factory AttributeValue.boolean({
    required bool value,
  }) = AttributeValue_Boolean;
  const factory AttributeValue.number({
    required PlatformInt64 value,
  }) = AttributeValue_Number;
}

class DisplayMetadata {
  final String lang;
  final String name;
  final String? description;
  final RenderingMetadata? rendering;

  const DisplayMetadata({
    required this.lang,
    required this.name,
    this.description,
    this.rendering,
  });

  @override
  int get hashCode => lang.hashCode ^ name.hashCode ^ description.hashCode ^ rendering.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is DisplayMetadata &&
          runtimeType == other.runtimeType &&
          lang == other.lang &&
          name == other.name &&
          description == other.description &&
          rendering == other.rendering;
}

class LogoMetadata {
  final String uri;
  final String uriIntegrity;
  final String altText;

  const LogoMetadata({
    required this.uri,
    required this.uriIntegrity,
    required this.altText,
  });

  @override
  int get hashCode => uri.hashCode ^ uriIntegrity.hashCode ^ altText.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is LogoMetadata &&
          runtimeType == other.runtimeType &&
          uri == other.uri &&
          uriIntegrity == other.uriIntegrity &&
          altText == other.altText;
}

@freezed
sealed class RenderingMetadata with _$RenderingMetadata {
  const RenderingMetadata._();

  const factory RenderingMetadata.simple({
    LogoMetadata? logo,
    String? backgroundColor,
    String? textColor,
  }) = RenderingMetadata_Simple;
  const factory RenderingMetadata.svgTemplates() = RenderingMetadata_SvgTemplates;
}
