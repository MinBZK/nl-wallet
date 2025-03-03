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
import '../default_text_and_focus_style.dart';
import '../organization/organization_logo.dart';

const _kThumbnailSize = 40.0;

class WalletEventRow extends StatefulWidget {
  final WalletEvent event;
  final VoidCallback onPressed;

  const WalletEventRow({
    required this.event,
    required this.onPressed,
    super.key,
  });

  @override
  State<WalletEventRow> createState() => _WalletEventRowState();
}

class _WalletEventRowState extends State<WalletEventRow> {
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(() => setState(() {})));
  }

  @override
  void dispose() {
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final Color? textPressedColor =
        context.theme.textButtonTheme.style?.foregroundColor?.resolve({WidgetState.pressed});
    final String titleText = WalletEventTitleFormatter.format(context, widget.event);
    final String timeAgoText = TimeAgoFormatter.format(context, widget.event.dateTime);
    final IconData? errorStatusIcon = WalletEventStatusIconMapper().map(widget.event);
    final String typeText = WalletEventStatusTextMapper().map(context, widget.event);
    final Color typeTextColor = WalletEventStatusColorMapper().map(context, widget.event);

    return TextButton.icon(
      onPressed: widget.onPressed,
      icon: const Icon(Icons.chevron_right),
      iconAlignment: IconAlignment.end,
      statesController: _statesController,
      style: context.theme.iconButtonTheme.style?.copyWith(
        shape: WidgetStateProperty.all(
          const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
        ),
      ),
      label: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 16),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.start,
              mainAxisSize: MainAxisSize.max,
              children: [
                ExcludeSemantics(
                  child: _buildThumbnail(context, widget.event),
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
                          child: DefaultTextAndFocusStyle(
                            statesController: _statesController,
                            textStyle: context.textTheme.titleMedium,
                            pressedOrFocusedColor: textPressedColor,
                            child: Text.rich(
                              titleText.toTextSpan(context),
                            ),
                          ),
                        ),
                      ),
                      DefaultTextAndFocusStyle(
                        statesController: _statesController,
                        textStyle: context.textTheme.bodyLarge?.copyWith(
                          color: typeTextColor,
                        ),
                        pressedOrFocusedColor: textPressedColor,
                        child: _buildTypeRow(
                          context,
                          errorStatusIcon,
                          typeText,
                        ),
                      ),
                      DefaultTextAndFocusStyle(
                        statesController: _statesController,
                        textStyle: context.textTheme.bodySmall,
                        pressedOrFocusedColor: textPressedColor,
                        child: Text.rich(
                          timeAgoText.toTextSpan(context),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
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

  Widget _buildTypeRow(BuildContext context, IconData? icon, String text) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 2),
      child: Row(
        children: [
          if (icon != null) ...[
            Icon(icon, color: context.colorScheme.error, size: 16),
            const SizedBox(width: 8),
          ],
          Flexible(
            child: Text.rich(
              text.toTextSpan(context),
            ),
          ),
        ],
      ),
    );
  }
}
