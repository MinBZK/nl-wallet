import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../feature/verification/model/organization.dart';
import '../../model/attribute/data_attribute.dart';
import '../../model/timeline_attribute.dart';

class LogCardInteractionUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  LogCardInteractionUseCase(this.timelineAttributeRepository);

  Future<void> invoke(
    InteractionType type,
    String cardId,
    Organization organization,
    List<DataAttribute> attributes,
  ) async {
    final interaction = InteractionAttribute(
      interactionType: type,
      dateTime: DateTime.now(),
      organization: organization,
      attributes: _getFilteredAttributes(type, attributes),
    );
    await timelineAttributeRepository.create(cardId, interaction);
  }

  /// Filters attributes for storage; only returns attributes for 'success' interaction
  List<DataAttribute> _getFilteredAttributes(InteractionType type, List<DataAttribute> attributes) {
    if (type == InteractionType.success) return attributes;
    return [];
  }
}
