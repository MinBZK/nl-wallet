import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../widget/loading_indicator.dart';

class GenericLoadingPage extends StatelessWidget {
  /// The title shown above the loading indicator
  final String title;

  /// The description shown above the loading indicator
  final String description;

  /// The action to perform when the cancel button is pressed, button is hidden when null
  final VoidCallback? onCancel;

  /// The text shown inside the cancel button, defaults to l10n.generalCancelCta
  final String? cancelCta;

  const GenericLoadingPage({
    required this.title,
    required this.description,
    this.onCancel,
    this.cancelCta,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Expanded(
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
        const LoadingIndicator(),
        Expanded(
          child: _buildOptionalCancelButton(context),
        ),
      ],
    );
  }

  Widget _buildOptionalCancelButton(BuildContext context) {
    if (onCancel == null) return const SizedBox.shrink();
    return Column(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        const SizedBox(height: 48),
        TextButton(
          onPressed: onCancel,
          child: Text(cancelCta ?? context.l10n.generalCancelCta),
        ),
      ],
    );
  }
}
