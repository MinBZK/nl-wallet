import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../widget/button/confirm/confirm_buttons.dart';
import '../widget/paragraphed_list.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_scrollbar.dart';

/// A terminal page layout that displays a title, description, an illustration,
/// and action buttons at the bottom.
///
/// This layout is typically used for success, error, or informational screens that
/// mark the end of a flow. Also see [TerminalScreen.show()] to use this as a route.
class TerminalPage extends StatelessWidget {
  /// The title displayed at the top of the page.
  final String title;

  /// An optional description text.
  ///
  /// If provided, the text will be split into paragraphs and displayed
  /// below the title using [ParagraphedList].
  final String? description;

  /// The primary action button.
  ///
  /// Usually a [PrimaryButton].
  final FitsWidthWidget? primaryButton;

  /// The secondary action button.
  ///
  /// Usually a [TertiaryButton].
  final FitsWidthWidget? secondaryButton;

  /// Whether buttons should be laid out vertically when both are present.
  ///
  /// This is ignored when the app is in landscape mode.
  final bool preferVerticalButtonLayout;

  /// An optional illustration widget.
  ///
  /// Usually a [PageIllustration].
  final Widget? illustration;

  const TerminalPage({
    required this.title,
    this.description,
    this.primaryButton,
    this.secondaryButton,
    this.illustration,
    this.preferVerticalButtonLayout = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      bottom: false,
      child: _buildScrollableSection(context),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: ConstrainedBox(
              constraints: BoxConstraints(minHeight: constraints.maxHeight),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  _buildContentSection(context),
                  _buildBottomSection(context),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildContentSection(BuildContext context) {
    return Column(
      children: [
        const SizedBox(height: 12),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: description == null
                ? [TitleText(title)]
                : [
                    TitleText(title),
                    const SizedBox(height: 8),
                    ParagraphedList.splitContent(description!),
                  ],
          ),
        ),
        const SizedBox(height: 24),
        ?illustration,
        SizedBox(height: illustration == null ? 0 : 24),
      ],
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    late Widget content;
    if (primaryButton != null && secondaryButton != null) {
      return Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Divider(),
          ConfirmButtons(
            primaryButton: primaryButton!,
            secondaryButton: secondaryButton!,
            forceVertical: preferVerticalButtonLayout && !context.isLandscape,
          ),
        ],
      );
    } else if (primaryButton != null) {
      content = primaryButton!;
    } else if (secondaryButton != null) {
      content = secondaryButton!;
    } else {
      return const SizedBox.shrink(); // Hide section
    }

    final contentPadding = context.isLandscape ? ConfirmButtons.contentLandscapePadding : ConfirmButtons.contentPadding;
    return SafeArea(
      top: false,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Divider(),
          Padding(
            padding: contentPadding,
            child: content,
          ),
        ],
      ),
    );
  }
}
