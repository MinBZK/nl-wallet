import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';

class LoginDetailScreenArgument {
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
}
