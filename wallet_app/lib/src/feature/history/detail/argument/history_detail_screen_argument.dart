class HistoryDetailScreenArgument {
  static const _kTimelineAttributeIdKey = 'timelineAttributeId';
  static const _kCardIdKey = 'cardId';
  static const _kCardDocType = 'docType';

  final String timelineAttributeId;
  final String? cardId;
  final String? docType;

  const HistoryDetailScreenArgument({required this.timelineAttributeId, this.cardId, this.docType});

  Map<String, dynamic> toMap() {
    return {
      _kTimelineAttributeIdKey: timelineAttributeId,
      _kCardIdKey: cardId,
      _kCardDocType: docType,
    };
  }

  static HistoryDetailScreenArgument fromMap(Map<String, dynamic> map) {
    return HistoryDetailScreenArgument(
      timelineAttributeId: map[_kTimelineAttributeIdKey],
      cardId: map[_kCardIdKey],
      docType: map[_kCardDocType],
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is HistoryDetailScreenArgument &&
          runtimeType == other.runtimeType &&
          timelineAttributeId == other.timelineAttributeId &&
          cardId == other.cardId &&
          docType == other.docType;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        timelineAttributeId,
        cardId,
        docType,
      );
}
