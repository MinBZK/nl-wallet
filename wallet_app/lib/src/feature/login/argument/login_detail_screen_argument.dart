import 'package:equatable/equatable.dart';

import '../../../domain/model/disclosure/disclose_card_request.dart';
import '../../../domain/model/organization.dart';

class LoginDetailScreenArgument extends Equatable {
  final Organization organization;
  final List<DiscloseCardRequest> cardRequests;
  final bool sharedDataWithOrganizationBefore;

  const LoginDetailScreenArgument({
    required this.organization,
    required this.cardRequests,
    required this.sharedDataWithOrganizationBefore,
  });

  @override
  List<Object?> get props => [organization, cardRequests, sharedDataWithOrganizationBefore];
}
