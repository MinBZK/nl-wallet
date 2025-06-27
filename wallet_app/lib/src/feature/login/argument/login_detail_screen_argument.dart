import 'package:equatable/equatable.dart';

import '../../../domain/model/disclosure/disclose_card_request.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';

class LoginDetailScreenArgument extends Equatable {
  final Organization organization;
  final Policy policy;
  final List<DiscloseCardRequest> cardRequests;
  final bool sharedDataWithOrganizationBefore;

  const LoginDetailScreenArgument({
    required this.organization,
    required this.policy,
    required this.cardRequests,
    required this.sharedDataWithOrganizationBefore,
  });

  @override
  List<Object?> get props => [organization, policy, cardRequests, sharedDataWithOrganizationBefore];
}
