import 'package:equatable/equatable.dart';

class Policy extends Equatable {
  final Duration? storageDuration;
  final String? dataPurpose;
  final bool dataIsShared;
  final bool dataIsSignature;
  final bool dataContainsSingleViewProfilePhoto;
  final bool deletionCanBeRequested;
  final String? privacyPolicyUrl;

  const Policy({
    required this.storageDuration,
    required this.dataPurpose,
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
        dataIsShared,
        dataIsSignature,
        dataContainsSingleViewProfilePhoto,
        deletionCanBeRequested,
        privacyPolicyUrl,
      ];
}
