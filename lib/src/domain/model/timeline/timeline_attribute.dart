import 'package:equatable/equatable.dart';

import '../../../feature/verification/model/organization.dart';
import '../attribute/data_attribute.dart';
import '../policy/policy.dart';

abstract class TimelineAttribute extends Equatable {
  final TimelineType type;
  final DateTime dateTime;
  final Organization organization;
  final List<DataAttribute> attributes;

  String get id => '${dateTime.microsecondsSinceEpoch.toString()}_${attributes.toString()}';

  const TimelineAttribute({
    required this.type,
    required this.dateTime,
    required this.organization,
    required this.attributes,
  });

  @override
  List<Object?> get props => [type, dateTime, organization, attributes];
}

enum TimelineType { interaction, operation, signing }

class InteractionAttribute extends TimelineAttribute {
  final InteractionStatus status;
  final Policy policy;

  const InteractionAttribute({
    required this.status,
    required this.policy,
    required super.dateTime,
    required super.organization,
    required super.attributes,
  }) : super(type: TimelineType.interaction);

  @override
  List<Object?> get props => [status, policy, ...super.props];
}

enum InteractionStatus { success, rejected, failed }

class OperationAttribute extends TimelineAttribute {
  final OperationStatus status;
  final String cardTitle;

  const OperationAttribute({
    required this.status,
    required this.cardTitle,
    required super.dateTime,
    required super.organization,
    required super.attributes,
  }) : super(type: TimelineType.operation);

  @override
  List<Object?> get props => [status, cardTitle, ...super.props];
}

enum OperationStatus { issued, renewed, expired }

class SigningAttribute extends TimelineAttribute {
  final SigningStatus status;
  final Policy policy;

  const SigningAttribute({
    required this.status,
    required this.policy,
    required super.dateTime,
    required super.organization,
    required super.attributes,
  }) : super(type: TimelineType.signing);

  @override
  List<Object?> get props => [status, ...super.props];
}

enum SigningStatus { success, rejected }
