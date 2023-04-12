import '../attribute/data_attribute.dart';
import '../policy/policy.dart';
import 'timeline_attribute.dart';

class InteractionTimelineAttribute extends TimelineAttribute {
  final InteractionStatus status;
  final Policy policy;

  const InteractionTimelineAttribute({
    required this.status,
    required this.policy,
    required super.dateTime,
    required super.organization,
    required super.dataAttributes,
  }) : super(type: TimelineType.interaction);

  @override
  List<Object?> get props => [status, policy, ...super.props];

  @override
  TimelineAttribute copyWith({List<DataAttribute>? dataAttributes}) {
    return InteractionTimelineAttribute(
      status: status,
      policy: policy,
      dateTime: dateTime,
      organization: organization,
      dataAttributes: dataAttributes ?? this.dataAttributes,
    );
  }
}

enum InteractionStatus { success, rejected, failed }
