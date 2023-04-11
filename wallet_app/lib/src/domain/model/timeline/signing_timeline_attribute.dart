import '../attribute/data_attribute.dart';
import '../document.dart';
import '../policy/policy.dart';
import 'timeline_attribute.dart';

class SigningTimelineAttribute extends TimelineAttribute {
  final SigningStatus status;
  final Policy policy;
  final Document document;

  const SigningTimelineAttribute({
    required this.status,
    required this.policy,
    required this.document,
    required super.dateTime,
    required super.organization,
    required super.dataAttributes,
  }) : super(type: TimelineType.signing);

  @override
  List<Object?> get props => [status, policy, document, ...super.props];

  @override
  TimelineAttribute copyWith({List<DataAttribute>? dataAttributes}) {
    return SigningTimelineAttribute(
      status: status,
      policy: policy,
      document: document,
      dateTime: dateTime,
      organization: organization,
      dataAttributes: dataAttributes ?? this.dataAttributes,
    );
  }
}

enum SigningStatus { success, rejected }
