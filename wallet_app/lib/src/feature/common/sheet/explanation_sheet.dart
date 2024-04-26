import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../widget/button/secondary_button.dart';

class ExplanationSheet extends StatelessWidget {
  final String title;
  final String description;
  final String closeButtonText;

  const ExplanationSheet({
    required this.title,
    required this.description,
    required this.closeButtonText,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      minimum: const EdgeInsets.symmetric(vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          MergeSemantics(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Text(
                    title,
                    style: context.textTheme.displayMedium,
                    textAlign: TextAlign.start,
                  ),
                ),
                const SizedBox(height: 16),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Text(
                    description,
                    style: context.textTheme.bodyLarge,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),
          const Divider(height: 1),
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: SecondaryButton(
              onPressed: () => Navigator.pop(context),
              text: closeButtonText,
              centerText: true,
              icon: Icons.close,
            ),
          ),
        ],
      ),
    );
  }

  static Future<void> show(
    BuildContext context, {
    required String title,
    required String description,
    required String closeButtonText,
  }) async {
    return showModalBottomSheet<void>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return Scrollbar(
          trackVisibility: true,
          child: SingleChildScrollView(
            child: ExplanationSheet(
              title: title,
              description: description,
              closeButtonText: closeButtonText,
            ),
          ),
        );
      },
    );
  }
}
