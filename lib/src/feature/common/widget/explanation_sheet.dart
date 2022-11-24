import 'package:flutter/material.dart';

import 'bottom_sheet_drag_handle.dart';

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
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          const Center(child: BottomSheetDragHandle()),
          const SizedBox(height: 24),
          Text(
            title,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 16),
          Text(
            description,
            style: Theme.of(context).textTheme.bodyText1,
          ),
          const SizedBox(height: 16),
          Center(
            child: TextButton(
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
      builder: (BuildContext context) {
        return ExplanationSheet(
          title: title,
          description: description,
          closeButtonText: closeButtonText,
        );
      },
    );
  }
}
