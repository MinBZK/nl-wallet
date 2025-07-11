import '../../../domain/model/card/card_config.dart';
import '../mapper.dart';

/// Mapper that creates a [CardConfig] based on the provided attestationId
class CardConfigMapper extends Mapper<String /* attestationId */, CardConfig> {
  CardConfigMapper();

  @override
  CardConfig map(String input) => const CardConfig(
        updatable: false,
        removable: false,
      );
}
