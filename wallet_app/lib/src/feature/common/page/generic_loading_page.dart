import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../widget/button/list_button.dart';
import '../widget/loading_indicator.dart';
import '../widget/wallet_app_bar.dart';

class GenericLoadingPage extends StatelessWidget {
  /// The title shown above the loading indicator
  final String title;

  /// The description shown above the loading indicator
  final String description;

  /// The action to perform when the cancel button is pressed, button is hidden when null
  final VoidCallback? onCancel;

  /// The text shown inside the cancel button, defaults to l10n.generalCancelCta
  final String? cancelCta;

  /// Appbar (e.g. a [WalletAppBar]) to be shown at the top of the top of the loading page,
  /// useful when the loading page should also render the stepperProgress.
  final PreferredSizeWidget? appBar;

  final Widget loadingIndicator;

  const GenericLoadingPage({
    required this.title,
    required this.description,
    this.onCancel,
    this.cancelCta,
    this.appBar,
    this.loadingIndicator = const LoadingIndicator(),
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: appBar,
      body: SizedBox(
        width: double.infinity,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Expanded(
              child: SingleChildScrollView(
                reverse: true,
                /* make sure it's bottom aligned */
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.end,
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    Text(
                      title,
                      style: context.textTheme.headlineMedium,
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      description,
                      style: context.textTheme.bodyLarge,
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 24),
                  ],
                ),
              ),
            ),
            loadingIndicator,
            Expanded(
              child: _buildOptionalCancelButton(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildOptionalCancelButton(BuildContext context) {
    if (onCancel == null) return const SizedBox.shrink();
    return SafeArea(
      left: false,
      right: false,
      child: Align(
        alignment: Alignment.bottomCenter,
        child: ListButton(
          icon: const Icon(Icons.block_outlined),
          onPressed: onCancel,
          dividerSide: DividerSide.top,
          mainAxisAlignment: MainAxisAlignment.center,
          text: Text(cancelCta ?? context.l10n.generalCancelCta),
        ),
      ),
    );
  }
}
