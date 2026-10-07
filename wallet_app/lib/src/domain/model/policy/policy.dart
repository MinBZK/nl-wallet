import 'package:equatable/equatable.dart';

import '../../../feature/policy/policy_screen.dart';

class Policy extends Equatable {
  final String? dataPurpose;

  /// Optional custom description, shown on the [PolicyScreen].
  final String? dataPurposeDescription;
  final String? privacyPolicyUrl;

  const Policy({
    this.dataPurpose,
    this.dataPurposeDescription,
    required this.privacyPolicyUrl,
  });

  @override
  List<Object?> get props => [
    dataPurpose,
    dataPurposeDescription,
    privacyPolicyUrl,
  ];
}
