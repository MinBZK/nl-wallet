import '../../../domain/model/card/card_config.dart';
import '../mapper.dart';

/// Mapper that creates a [CardConfig] based on the provided docType
class CardConfigMapper extends Mapper<String /* docType */, CardConfig> {
  CardConfigMapper();

  @override
  CardConfig map(String input) => const CardConfig(
        updatable: false,
        removable: false,
      );
}
