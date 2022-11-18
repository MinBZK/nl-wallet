part of 'mock_timeline_attribute_repository.dart';

final List<TimelineAttribute> _kMockCardIdPidOneUsageAttributes = [
  InteractionAttribute(
    interactionType: InteractionType.success,
    organization: 'Amsterdam Airport Schiphol',
    dateTime: DateTime.now().subtract(const Duration(hours: 4)),
  ),
  OperationAttribute(
    operationType: OperationType.issued,
    description: 'Deze kaart is geldig tot 12 oktober 2025',
    dateTime: DateTime.now().subtract(const Duration(days: 5)),
  ),
];

final List<TimelineAttribute> _kMockCardIdOneUsageAttributes = [
  OperationAttribute(
    operationType: OperationType.extended,
    description: 'Deze kaart is geldig tot 12 oktober 2025',
    dateTime: DateTime.now().subtract(const Duration(minutes: 11)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.success,
    organization: 'Organisatie X',
    dateTime: DateTime.now().subtract(const Duration(minutes: 57)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.success,
    organization: 'Organisatie Y',
    dateTime: DateTime.now().subtract(const Duration(hours: 2)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Organisatie Z',
    dateTime: DateTime.now().subtract(const Duration(hours: 13)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.failed,
    organization: 'Organisatie A',
    dateTime: DateTime.now().subtract(const Duration(days: 2)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.success,
    organization: 'Organisatie B',
    dateTime: DateTime.now().subtract(const Duration(days: 8)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Organisatie C',
    dateTime: DateTime.now().subtract(const Duration(days: 35)),
  ),
];

final List<InteractionAttribute> _kMockCardIdTwoUsageAttributes = [
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Organisatie K',
    dateTime: DateTime.now().subtract(const Duration(minutes: 2)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Organisatie L',
    dateTime: DateTime.now().subtract(const Duration(hours: 3)),
  ),
];
