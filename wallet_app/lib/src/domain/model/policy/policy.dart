import 'package:equatable/equatable.dart';

import '../../../feature/policy/policy_screen.dart';

class Policy extends Equatable {
  final Duration? storageDuration;
  final String? dataPurpose;

  /// Optional custom description, shown on the [PolicyScreen].
  final String? dataPurposeDescription;
  final bool dataIsShared;
  final bool deletionCanBeRequested;
  final String? privacyPolicyUrl;

  /// FIXME: Remove [dataIsSignature] at some point, only relevant for mock build.
  final bool dataIsSignature;

  /// FIXME: Remove [dataContainsSingleViewProfilePhoto] at some point, only relevant for mock build.
  final bool dataContainsSingleViewProfilePhoto;

  const Policy({
    this.storageDuration,
    this.dataPurpose,
    this.dataPurposeDescription,
    required this.dataIsShared,
    required this.dataIsSignature,
    required this.dataContainsSingleViewProfilePhoto,
    required this.deletionCanBeRequested,
    required this.privacyPolicyUrl,
  });

  @override
  List<Object?> get props => [
        storageDuration,
        dataPurpose,
        dataPurposeDescription,
        dataIsShared,
        dataIsSignature,
        dataContainsSingleViewProfilePhoto,
        deletionCanBeRequested,
        privacyPolicyUrl,
      ];
}
