import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../model/timeline_attribute.dart';

class LogCardInteractionUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  LogCardInteractionUseCase(this.timelineAttributeRepository);

  Future<void> invoke(String cardId, InteractionType type, String organizationName) async {
    final interaction = InteractionAttribute(
      interactionType: type,
      organization: organizationName,
      dateTime: DateTime.now(),
    );
    await timelineAttributeRepository.create(cardId, interaction);
  }
}
