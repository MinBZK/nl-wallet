import 'package:flutter/material.dart';

import '../../../../domain/model/event/wallet_event.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/formatter/history_details_time_formatter.dart';
import '../../../../util/mapper/event/wallet_event_status_color_mapper.dart';
import '../../../../util/mapper/event/wallet_event_status_description_mapper.dart';
import '../../../../util/mapper/event/wallet_event_status_icon_mapper.dart';
import '../../../../util/mapper/event/wallet_event_status_title_mapper.dart';

const _kStatusIconSize = 24.0;

class WalletEventStatusHeader extends StatelessWidget {
  final WalletEvent event;

  const WalletEventStatusHeader({
    required this.event,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final String titleText = WalletEventStatusTitleMapper().map(context, event);
    final String descriptionText = WalletEventStatusDescriptionMapper().map(context, event);
    final IconData? errorStatusIcon = WalletEventStatusIconMapper().map(event);
    final Color statusColor = WalletEventStatusColorMapper().map(context, event);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              if (errorStatusIcon != null)
                Icon(
                  errorStatusIcon,
                  color: statusColor,
                  size: _kStatusIconSize,
                )
              else
                const SizedBox(
                  width: _kStatusIconSize,
                  height: _kStatusIconSize,
                ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Text(
                      titleText,
                      style: context.textTheme.titleMedium,
                    ),
                    const SizedBox(height: 2),
                    Text(
                      HistoryDetailsTimeFormatter.format(context, event.dateTime),
                      style: context.textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 24),
          Text(
            descriptionText,
            style: context.textTheme.bodyLarge,
          ),
        ],
      ),
    );
  }
}
