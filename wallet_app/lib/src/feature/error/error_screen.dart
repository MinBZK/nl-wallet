import 'package:flutter/material.dart';

import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/text_icon_button.dart';
import '../common/widget/sliver_sized_box.dart';

class ErrorScreen extends StatelessWidget {
  final String title;
  final String headline;
  final String description;
  final String primaryActionText;
  final String? secondaryActionText;
  final VoidCallback onPrimaryActionPressed;
  final VoidCallback? onSecondaryActionPressed;

  const ErrorScreen({
    required this.title,
    required this.headline,
    required this.description,
    required this.primaryActionText,
    required this.onPrimaryActionPressed,
    this.secondaryActionText,
    this.onSecondaryActionPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(title),
        leading: const SizedBox.shrink(),
        actions: const [CloseButton()],
      ),
      body: PrimaryScrollController(
        controller: ScrollController(),
        child: Scrollbar(
          thumbVisibility: true,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: CustomScrollView(
              slivers: [
                const SliverSizedBox(height: 24),
                SliverToBoxAdapter(
                  child: Container(
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: const Color(0xFFF5F5FD),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    width: double.infinity,
                    height: 100,
                    child: const Text('Placeholder image'),
                  ),
                ),
                const SliverSizedBox(height: 24),
                SliverToBoxAdapter(
                  child: Text(
                    headline,
                    textAlign: TextAlign.start,
                    style: context.textTheme.displayMedium,
                  ),
                ),
                const SliverSizedBox(height: 8),
                SliverToBoxAdapter(
                  child: Text(
                    description,
                    textAlign: TextAlign.start,
                    style: context.textTheme.bodyLarge,
                  ),
                ),
                const SliverSizedBox(height: 32),
                SliverFillRemaining(
                  hasScrollBody: false,
                  fillOverscroll: true,
                  child: _buildBottomSection(context),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        ElevatedButton(
          onPressed: onPrimaryActionPressed,
          child: Text(primaryActionText),
        ),
        if (secondaryActionText != null) ...[
          const SizedBox(height: 8),
          Center(
            child: TextIconButton(
              onPressed: onSecondaryActionPressed,
              child: Text(secondaryActionText!),
            ),
          ),
        ],
        const SizedBox(height: 24),
      ],
    );
  }

  static void show(
    BuildContext context, {
    bool secured = true,
    required String title,
    required String headline,
    required String description,
    required String primaryActionText,
    required VoidCallback onPrimaryActionPressed,
    String? secondaryActionText,
    VoidCallback? onSecondaryActionPressed,
  }) {
    Navigator.push(
      context,
      secured
          ? SecuredPageRoute(
              builder: (c) => ErrorScreen(
                title: title,
                headline: headline,
                description: description,
                primaryActionText: primaryActionText,
                onPrimaryActionPressed: onPrimaryActionPressed,
                secondaryActionText: secondaryActionText,
                onSecondaryActionPressed: onSecondaryActionPressed,
              ),
            )
          : MaterialPageRoute(
              builder: (c) => ErrorScreen(
                title: title,
                headline: headline,
                description: description,
                primaryActionText: primaryActionText,
                onPrimaryActionPressed: onPrimaryActionPressed,
                secondaryActionText: secondaryActionText,
                onSecondaryActionPressed: onSecondaryActionPressed,
              ),
            ),
    );
  }

  /// Shows the [ErrorScreen] with the most generic error message
  /// i.e. 'something went wrong' and a close button. Useful when
  /// we only want to communicate something went wrong without going
  /// into any specifics.
  static void showGeneric(BuildContext context, {bool secured = true}) {
    show(
      context,
      secured: secured,
      title: context.l10n.errorScreenGenericTitle,
      headline: context.l10n.errorScreenGenericHeadline,
      description: context.l10n.errorScreenGenericDescription,
      primaryActionText: context.l10n.errorScreenGenericCloseCta,
      onPrimaryActionPressed: () => Navigator.pop(context),
    );
  }
}
