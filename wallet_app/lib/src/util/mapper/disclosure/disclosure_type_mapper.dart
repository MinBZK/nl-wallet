import 'package:wallet_core/core.dart' as core show DisclosureType;

import '../../../domain/model/disclosure/disclosure_type.dart';
import '../mapper.dart';

class DisclosureTypeMapper extends Mapper<core.DisclosureType, DisclosureType> {
  DisclosureTypeMapper();

  @override
  DisclosureType map(core.DisclosureType input) {
    return switch (input) {
      core.DisclosureType.Regular => DisclosureType.regular,
      core.DisclosureType.Login => DisclosureType.login,
    };
  }
}
