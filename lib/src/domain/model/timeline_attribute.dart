import 'package:equatable/equatable.dart';

abstract class TimelineAttribute extends Equatable {
  final DateTime dateTime;
  final TimelineType timelineType;

  const TimelineAttribute(this.dateTime, this.timelineType);
}

enum TimelineType { operation, interaction }

class InteractionAttribute extends TimelineAttribute {
  final InteractionType interactionType;
  final String organization;

  const InteractionAttribute({
    required this.interactionType,
    required this.organization,
    required DateTime dateTime,
  }) : super(dateTime, TimelineType.interaction);

  @override
  List<Object?> get props => [interactionType, organization, dateTime];
}

enum InteractionType { success, rejected, failed }

class OperationAttribute extends TimelineAttribute {
  final OperationType operationType;
  final String cardTitle;

  const OperationAttribute({
    required this.operationType,
    required this.cardTitle,
    required DateTime dateTime,
  }) : super(dateTime, TimelineType.operation);

  @override
  List<Object?> get props => [operationType, cardTitle, dateTime];
}

enum OperationType { issued, extended, expired }
