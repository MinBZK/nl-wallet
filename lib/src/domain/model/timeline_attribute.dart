import 'package:equatable/equatable.dart';

import '../../feature/verification/model/organization.dart';
import 'attribute/data_attribute.dart';

abstract class TimelineAttribute extends Equatable {
  final TimelineType timelineType;
  final DateTime dateTime;
  final Organization organization;
  final List<DataAttribute> attributes;

  String get id => '${dateTime.microsecondsSinceEpoch.toString()}_${attributes.toString()}';

  const TimelineAttribute(this.timelineType, this.dateTime, this.organization, this.attributes);

  @override
  List<Object?> get props => [timelineType, dateTime, organization, attributes];
}

enum TimelineType { interaction, operation }

class InteractionAttribute extends TimelineAttribute {
  final InteractionType interactionType;

  const InteractionAttribute({
    required this.interactionType,
    required DateTime dateTime,
    required Organization organization,
    required List<DataAttribute> attributes,
  }) : super(TimelineType.interaction, dateTime, organization, attributes);

  @override
  List<Object?> get props => [interactionType, ...super.props];
}

enum InteractionType { success, rejected, failed }

class OperationAttribute extends TimelineAttribute {
  final OperationType operationType;
  final String cardTitle;

  const OperationAttribute({
    required this.operationType,
    required this.cardTitle,
    required DateTime dateTime,
    required Organization organization,
    required List<DataAttribute> attributes,
  }) : super(TimelineType.operation, dateTime, organization, attributes);

  @override
  List<Object?> get props => [operationType, cardTitle, ...super.props];
}

enum OperationType { issued, renewed, expired }
