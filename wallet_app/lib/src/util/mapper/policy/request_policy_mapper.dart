import 'package:wallet_core/core.dart';

import '../../../domain/model/policy/policy.dart';
import '../mapper.dart';

class RequestPolicyMapper extends Mapper<RequestPolicy, Policy> {
  RequestPolicyMapper();

  @override
  Policy map(RequestPolicy input) => Policy(
        storageDuration: input.dataStorageDurationInMinutes == null
            ? null
            : Duration(minutes: input.dataStorageDurationInMinutes!.toInt()), //todo: don't use BigInt
        dataIsShared: input.dataSharedWithThirdParties,
        deletionCanBeRequested: input.dataDeletionPossible,
        privacyPolicyUrl: input.policyUrl,
      );
}
