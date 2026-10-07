import 'package:wallet_core/core.dart';

import '/src/domain/model/policy/policy.dart';
import '/src/util/mapper/mapper.dart';

class RequestPolicyMapper extends Mapper<RequestPolicy, Policy> {
  RequestPolicyMapper();

  @override
  Policy map(RequestPolicy input) => Policy(
    privacyPolicyUrl: input.policyUrl,
  );
}
