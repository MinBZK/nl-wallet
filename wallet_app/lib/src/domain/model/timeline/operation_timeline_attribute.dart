import '../attribute/data_attribute.dart';
import '../wallet_card.dart';
import 'timeline_attribute.dart';

class OperationTimelineAttribute extends TimelineAttribute {
  final OperationStatus status;
  final WalletCard card;

  const OperationTimelineAttribute({
    required this.status,
    required this.card,
    required super.dateTime,
    required super.organization,
    required super.dataAttributes,
  }) : super(type: TimelineType.operation);

  @override
  List<Object?> get props => [status, card, ...super.props];

  @override
  TimelineAttribute copyWith({List<DataAttribute>? dataAttributes, OperationStatus? status, DateTime? dateTime}) {
    return OperationTimelineAttribute(
      status: status ?? this.status,
      card: card,
      dateTime: dateTime ?? this.dateTime,
      organization: organization,
      dataAttributes: dataAttributes ?? this.dataAttributes,
    );
  }
}

enum OperationStatus { issued, renewed, expired }
