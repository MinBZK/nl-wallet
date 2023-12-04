import '../../../../domain/model/timeline/timeline_attribute.dart';

class HistoryDetailScreenArgument {
  static const _kTimelineAttributeKey = 'timelineAttribute';
  static const _kCardDocType = 'docType';

  final TimelineAttribute timelineAttribute;
  final String? docType;

  const HistoryDetailScreenArgument({required this.timelineAttribute, this.docType});

  Map<String, dynamic> toMap() {
    return {
      _kTimelineAttributeKey: timelineAttribute,
      _kCardDocType: docType,
    };
  }

  static HistoryDetailScreenArgument fromMap(Map<String, dynamic> map) {
    return HistoryDetailScreenArgument(
      timelineAttribute: map[_kTimelineAttributeKey],
      docType: map[_kCardDocType],
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is HistoryDetailScreenArgument &&
          runtimeType == other.runtimeType &&
          timelineAttribute == other.timelineAttribute &&
          docType == other.docType;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        timelineAttribute,
        docType,
      );
}
