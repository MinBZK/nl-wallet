import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'button/text_icon_button.dart';

class ExplanationSheet extends StatelessWidget {
  final String title;
  final String description;
  final String closeButtonText;

  const ExplanationSheet({
    required this.title,
    required this.description,
    required this.closeButtonText,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      minimum: const EdgeInsets.only(bottom: 16),
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
          const Divider(height: 32),
          Center(
            child: TextIconButton(
              icon: Icons.close,
              iconPosition: IconPosition.start,
              child: Text(closeButtonText),
              onPressed: () => Navigator.pop(context),
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
