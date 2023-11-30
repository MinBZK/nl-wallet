import '../../../domain/model/card_config.dart';
import '../mapper.dart';

/// Mapper that creates a [CardConfig] based on the provided docType, relies
/// on knowledge about the mock.
class CardConfigMapper extends Mapper<String /* docType */, CardConfig> {
  CardConfigMapper();

  @override
  CardConfig map(String input) => const CardConfig(
        updatable: false, // Can be set to 'input == kDrivingLicenseDocType' to enable updating of driving license.
        removable: false,
      );
}
