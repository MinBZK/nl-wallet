import 'package:wallet_core/core.dart';

import '/src/domain/model/policy/policy.dart';
import '/src/util/extension/object_extension.dart';
import '/src/util/mapper/mapper.dart';

class RequestPolicyMapper extends Mapper<RequestPolicy, Policy> {
  RequestPolicyMapper();

  @override
  Policy map(RequestPolicy input) => Policy(
        storageDuration: input.dataStorageDurationInMinutes?.let((it) => Duration(minutes: it.toInt())),
        dataIsShared: input.dataSharedWithThirdParties,
        deletionCanBeRequested: input.dataDeletionPossible,
        privacyPolicyUrl: input.policyUrl,
      );
}
