import 'package:flutter/material.dart';

import '../../../../util/formatter/history_details_time_formatter.dart';

class HistoryDetailTimestamp extends StatelessWidget {
  final DateTime dateTime;
  final EdgeInsets padding;

  const HistoryDetailTimestamp({
    required this.dateTime,
    this.padding = const EdgeInsets.symmetric(horizontal: 16),
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: padding,
      child: Text.rich(
        HistoryDetailsTimeFormatter.format(context, dateTime),
      ),
    );
  }
}
