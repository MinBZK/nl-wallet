import 'package:equatable/equatable.dart';

class VerifierPolicy extends Equatable {
  final Duration storageDuration;
  final String dataPurpose;
  final bool dataIsShared;
  final bool deletionCanBeRequested;
  final String privacyPolicyUrl;

  const VerifierPolicy({
    required this.storageDuration,
    required this.dataPurpose,
    required this.dataIsShared,
    required this.deletionCanBeRequested,
    required this.privacyPolicyUrl,
  });

  @override
  List<Object?> get props => [storageDuration, dataPurpose, dataIsShared, deletionCanBeRequested, privacyPolicyUrl];
}
