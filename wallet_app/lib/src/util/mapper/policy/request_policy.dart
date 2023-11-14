import '../../../domain/model/policy/policy.dart';
import '../../../wallet_core/wallet_core.dart';
import '../mapper.dart';

class RequestPolicyMapper extends Mapper<RequestPolicy, Policy> {
  RequestPolicyMapper();

  @override
  Policy map(RequestPolicy input) => Policy(
        storageDuration: input.dataStorageDurationDays == null ? null : Duration(days: input.dataStorageDurationDays!),
        dataIsShared: input.dataSharedWithThirdParties,
        dataIsSignature: false,
        dataContainsSingleViewProfilePhoto: false,
        deletionCanBeRequested: input.dataDeletionPossible,
        privacyPolicyUrl: input.policyUrl,
      );
}
