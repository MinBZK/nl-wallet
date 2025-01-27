import 'package:equatable/equatable.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';

class LoginDetailScreenArgument extends Equatable {
  final Organization organization;
  final Policy policy;
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;
  final bool sharedDataWithOrganizationBefore;

  const LoginDetailScreenArgument({
    required this.organization,
    required this.policy,
    required this.requestedAttributes,
    required this.sharedDataWithOrganizationBefore,
  });

  @override
  List<Object?> get props => [organization, policy, requestedAttributes, sharedDataWithOrganizationBefore];
}
