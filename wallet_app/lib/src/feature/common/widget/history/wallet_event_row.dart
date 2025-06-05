import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../../domain/model/event/wallet_event.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../util/extension/wallet_event_extension.dart';
import '../../../../util/formatter/wallet_event_title_formatter.dart';
import '../../../../util/mapper/event/wallet_event_status_color_mapper.dart';
import '../../../../util/mapper/event/wallet_event_status_icon_mapper.dart';
import '../../../../util/mapper/event/wallet_event_status_text_mapper.dart';
import '../card/wallet_card_item.dart';
import '../menu_item.dart';
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
    final String titleText = WalletEventTitleFormatter.format(context, event);
    final DateFormat dateFormat = DateFormat(DateFormat.MONTH_DAY, context.l10n.localeName);
    final String formattedTime = dateFormat.format(event.dateTime);
    final IconData? errorStatusIcon = WalletEventStatusIconMapper().map(event);
    final String typeText = WalletEventStatusTextMapper().map(context, event);
    final Color? typeTextColor = WalletEventStatusColorMapper().useErrorColor(event) ? context.colorScheme.error : null;

    return MenuItem(
      label: Text.rich(titleText.toTextSpan(context)),
      subtitle: Builder(
        builder: (context) {
          // Resolve DefaultTextStyle using builder to inherit the underline behaviour and color
          return Text.rich(
            typeText.toTextSpan(context),
            style: DefaultTextStyle.of(context).style.copyWith(color: typeTextColor),
          );
        },
      ),
      underline: Text.rich(formattedTime.toTextSpan(context)),
      largeIcon: true,
      leftIcon: ExcludeSemantics(child: _buildThumbnail(context, event)),
      errorIcon: errorStatusIcon == null ? null : Icon(errorStatusIcon),
      onPressed: onPressed,
    );
  }

  /// For card related operations (issued/renewed/expired) show the card as a thumbnail,
  /// otherwise show the organization logo as the thumbnail.
  Widget _buildThumbnail(BuildContext context, WalletEvent event) {
    if (event is IssuanceEvent) {
      return SizedBox(
        width: _kThumbnailSize,
        child: WalletCardItem.fromWalletCard(
          context,
          event.card,
          scaleText: false,
          showText: false,
        ),
      );
    }

    return OrganizationLogo(
      image: event.relyingPartyOrIssuer.logo,
      size: _kThumbnailSize,
    );
  }
}
