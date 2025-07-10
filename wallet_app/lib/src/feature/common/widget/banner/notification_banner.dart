import 'package:flutter/material.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../theme/wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../text/body_text.dart';

/// A reusable notification banner widget with configurable content and styling.
///
/// This widget displays a banner with a leading icon, a title, an optional
/// subtitle, and a trailing arrow icon. It is designed to be interactive,
/// triggering an `onTap` callback when pressed. The banner's appearance
/// changes slightly on focus to provide visual feedback.
class NotificationBanner extends StatefulWidget {
  /// The widget to display at the beginning of the banner.
  ///
  /// Typically an [Icon] widget.
  final Widget leadingIcon;

  /// The main text content of the banner.
  final String title;

  /// Optional additional text content displayed below the title.
  final String? subtitle;

  /// Callback function invoked when the banner is tapped.
  final VoidCallback onTap;

  const NotificationBanner({
    required this.leadingIcon,
    required this.title,
    required this.onTap,
    this.subtitle,
    super.key,
  });

  @override
  State<NotificationBanner> createState() => _NotificationBannerState();
}

class _NotificationBannerState extends State<NotificationBanner> {
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(_updateState));
  }

  void _updateState() => setState(() {});

  @override
  void dispose() {
    _statesController.removeListener(_updateState);
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MergeSemantics(
      child: Material(
        color: context.colorScheme.tertiaryContainer,
        shape: RoundedRectangleBorder(
          borderRadius: WalletTheme.kBorderRadius12,
          side: context.theme.elevatedButtonTheme.style?.side?.resolve(_statesController.value) ?? BorderSide.none,
        ),
        child: InkWell(
          statesController: _statesController,
          borderRadius: WalletTheme.kBorderRadius12,
          onTap: widget.onTap,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                widget.leadingIcon,
                const SizedBox(width: 12),
                Expanded(child: _buildTextContent(context)),
                const SizedBox(width: 12),
                _buildForwardIcon(),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildTextContent(BuildContext context) {
    final title = BodyText(
      widget.title,
      style: context.textTheme.titleMedium?.copyWith(
        decoration: _statesController.value.isFocused ? TextDecoration.underline : null,
      ),
    );

    if (widget.subtitle == null) return title;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        title,
        BodyText(
          widget.subtitle!,
          style: context.textTheme.bodyLarge?.copyWith(
            decoration: _statesController.value.isFocused ? TextDecoration.underline : null,
          ),
        ),
      ],
    );
  }

  Widget _buildForwardIcon() {
    return Icon(
      Icons.arrow_forward_outlined,
      size: context.theme.iconTheme.size! * (_statesController.value.isFocused ? 1.25 : 1.0),
    );
  }
}
