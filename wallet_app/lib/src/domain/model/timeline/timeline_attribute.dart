import 'package:equatable/equatable.dart';

import '../../../feature/verification/model/organization.dart';
import '../attribute/data_attribute.dart';

abstract class TimelineAttribute extends Equatable {
  final TimelineType type;
  final DateTime dateTime;
  final Organization organization;
  final List<DataAttribute> dataAttributes;

  String get id => dateTime.microsecondsSinceEpoch.toString();

  const TimelineAttribute({
    required this.type,
    required this.dateTime,
    required this.organization,
    required this.dataAttributes,
  });

  @override
  List<Object?> get props => [type, dateTime, organization, dataAttributes];

  TimelineAttribute copyWith({List<DataAttribute>? dataAttributes});
}

enum TimelineType { interaction, operation, signing }
