import 'package:flutter/material.dart';

import '../../../../domain/model/event/wallet_event.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
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

    Widget? icon;
    if (errorStatusIcon != null) {
      icon = Icon(
        errorStatusIcon,
        color: statusColor,
        size: _kStatusIconSize,
      );
    }

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          icon ?? const SizedBox.shrink(),
          SizedBox(height: icon == null ? 0 : 16),
          MergeSemantics(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text.rich(
                  titleText.toTextSpan(context),
                  style: context.textTheme.titleMedium?.copyWith(color: statusColor),
                ),
                const SizedBox(height: 8),
                Text.rich(
                  descriptionText.toTextSpan(context),
                  style: context.textTheme.bodyLarge,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
