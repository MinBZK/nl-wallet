class HistoryDetailScreenArgument {
  static const _kTimelineAttributeIdKey = 'timelineAttributeId';
  static const _kCardIdKey = 'cardId';

  final String timelineAttributeId;
  final String? cardId;

  const HistoryDetailScreenArgument({required this.timelineAttributeId, this.cardId});

  Map<String, dynamic> toMap() {
    return {
      _kTimelineAttributeIdKey: timelineAttributeId,
      _kCardIdKey: cardId,
    };
  }

  static HistoryDetailScreenArgument fromMap(Map<String, dynamic> map) {
    return HistoryDetailScreenArgument(
      timelineAttributeId: map[_kTimelineAttributeIdKey],
      cardId: map[_kCardIdKey],
    );
  }
}
