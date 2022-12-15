import 'package:equatable/equatable.dart';

class InteractionPolicy extends Equatable {
  final Duration? storageDuration;
  final String? dataPurpose;
  final bool dataIsShared;
  final bool dataIsSignature;
  final bool deletionCanBeRequested;
  final String? privacyPolicyUrl;

  const InteractionPolicy({
    required this.storageDuration,
    required this.dataPurpose,
    required this.dataIsShared,
    required this.dataIsSignature,
    required this.deletionCanBeRequested,
    required this.privacyPolicyUrl,
  });

  @override
  List<Object?> get props => [
        storageDuration,
        dataPurpose,
        dataIsShared,
        dataIsSignature,
        deletionCanBeRequested,
        privacyPolicyUrl,
      ];
}
