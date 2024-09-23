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

  bool get dataIsStored => storageDuration != null;

  const Policy({
    this.storageDuration,
    this.dataPurpose,
    this.dataPurposeDescription,
    required this.dataIsShared,
    required this.deletionCanBeRequested,
    required this.privacyPolicyUrl,
  });

  @override
  List<Object?> get props => [
        storageDuration,
        dataPurpose,
        dataPurposeDescription,
        dataIsShared,
        deletionCanBeRequested,
        privacyPolicyUrl,
      ];
}
