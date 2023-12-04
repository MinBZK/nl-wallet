import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';

import '../attribute/data_attribute.dart';
import '../organization.dart';

abstract class TimelineAttribute extends Equatable {
  final TimelineType type;
  final DateTime dateTime;
  final Organization organization;
  final List<DataAttribute> dataAttributes;

  const TimelineAttribute({
    required this.type,
    required this.dateTime,
    required this.organization,
    required this.dataAttributes,
  });

  Map<String, List<DataAttribute>> get attributesByDocType => groupBy(dataAttributes, (attr) => attr.sourceCardDocType);

  @override
  List<Object?> get props => [type, dateTime, organization, dataAttributes];

  TimelineAttribute copyWith({List<DataAttribute>? dataAttributes});
}

enum TimelineType { interaction, operation, signing }
