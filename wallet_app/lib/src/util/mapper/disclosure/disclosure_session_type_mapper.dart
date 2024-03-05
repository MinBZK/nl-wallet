import 'package:wallet_core/core.dart' as core show DisclosureSessionType;

import '../../../domain/model/disclosure/disclosure_session_type.dart';
import '../mapper.dart';

class DisclosureSessionTypeMapper extends Mapper<core.DisclosureSessionType, DisclosureSessionType> {
  DisclosureSessionTypeMapper();

  @override
  DisclosureSessionType map(core.DisclosureSessionType input) {
    return switch (input) {
      core.DisclosureSessionType.SameDevice => DisclosureSessionType.sameDevice,
      core.DisclosureSessionType.CrossDevice => DisclosureSessionType.crossDevice,
    };
  }
}
