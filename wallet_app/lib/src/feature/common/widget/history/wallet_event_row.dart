import 'package:flutter/material.dart';

import '../../../../domain/model/event/wallet_event.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../util/extension/wallet_event_extension.dart';
import '../../../../util/formatter/time_ago_formatter.dart';
import '../../../../util/formatter/wallet_event_title_formatter.dart';
import '../../../../util/mapper/event/wallet_event_status_color_mapper.dart';
import '../../../../util/mapper/event/wallet_event_status_icon_mapper.dart';
import '../../../../util/mapper/event/wallet_event_status_text_mapper.dart';
import '../card/wallet_card_item.dart';
import '../organization/organization_logo.dart';

const _kThumbnailSize = 40.0;

class WalletEventRow extends StatelessWidget {
  final WalletEvent event;
  final VoidCallback onPressed;

  const WalletEventRow({
    required this.event,
    required this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final String titleText = WalletEventTitleFormatter.format(
      context,
      event,
    );
    final String timeAgoText = TimeAgoFormatter.format(
      context,
      event.dateTime,
    );

    return InkWell(
      onTap: onPressed,
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.start,
              mainAxisSize: MainAxisSize.max,
              children: [
                ExcludeSemantics(
                  child: _buildThumbnail(context, event),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Visibility(
                        visible: titleText.isNotEmpty,
                        child: Padding(
                          padding: const EdgeInsets.only(bottom: 2),
                          child: Text.rich(titleText.toTextSpan(context), style: context.textTheme.titleMedium),
                        ),
                      ),
                      _buildTypeRow(context, event),
                      Text.rich(timeAgoText.toTextSpan(context), style: context.textTheme.bodySmall),
                    ],
                  ),
                ),
                const SizedBox(width: 16),
                ExcludeSemantics(
                  child: Icon(
                    Icons.chevron_right,
                    color: context.colorScheme.onSurface,
                  ),
                ),
              ],
            ),
          ),
          const Divider(),
        ],
      ),
    );
  }

  /// For card related operations (issued/renewed/expired) show the card as a thumbnail,
  /// otherwise show the organization logo as the thumbnail.
  Widget _buildThumbnail(BuildContext context, WalletEvent event) {
    if (event is IssuanceEvent) {
      return SizedBox(
        width: _kThumbnailSize,
        child: WalletCardItem.fromCardFront(
          context: context,
          front: event.card.front,
          scaleText: false,
        ),
      );
    }

    return OrganizationLogo(
      image: event.relyingPartyOrIssuer.logo,
      size: _kThumbnailSize,
    );
  }

  Widget _buildTypeRow(BuildContext context, WalletEvent event) {
    final IconData? errorStatusIcon = WalletEventStatusIconMapper().map(event);
    final String typeText = WalletEventStatusTextMapper().map(context, event);
    final Color typeTextColor = WalletEventStatusColorMapper().map(context, event);

    return Padding(
      padding: const EdgeInsets.only(bottom: 2),
      child: Row(
        children: [
          if (errorStatusIcon != null) ...[
            Icon(errorStatusIcon, color: context.colorScheme.error, size: 16),
            const SizedBox(width: 8),
          ],
          Flexible(
            child: Text.rich(
              typeText.toTextSpan(context),
              style: context.textTheme.bodyLarge?.copyWith(color: typeTextColor),
            ),
          ),
        ],
      ),
    );
  }
}
