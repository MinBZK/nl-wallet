import 'package:flutter/material.dart';

import 'bottom_sheet_drag_handle.dart';
import 'confirm_buttons.dart';

class ConfirmActionSheet extends StatelessWidget {
  final VoidCallback? onCancel;
  final VoidCallback? onConfirm;
  final String title;
  final String description;
  final String cancelButtonText;
  final String confirmButtonText;
  final Color? confirmButtonColor;

  const ConfirmActionSheet({
    this.onCancel,
    this.onConfirm,
    this.confirmButtonColor,
    required this.title,
    required this.description,
    required this.cancelButtonText,
    required this.confirmButtonText,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: Theme.of(context).copyWith(elevatedButtonTheme: buttonTheme(context)),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            const Center(child: BottomSheetDragHandle()),
            const SizedBox(height: 24),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16.0),
              child: Text(
                title,
                style: Theme.of(context).textTheme.headline2,
                textAlign: TextAlign.start,
              ),
            ),
            const SizedBox(height: 16),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16.0),
              child: Text(
                description,
                style: Theme.of(context).textTheme.bodyText1,
              ),
            ),
            const SizedBox(height: 16),
            ConfirmButtons(
              onDecline: () => onCancel?.call(),
              onAccept: () => onConfirm?.call(),
              acceptText: confirmButtonText,
              acceptIcon: null,
              declineText: cancelButtonText,
              declineIcon: null,
            ),
          ],
        ),
      ),
    );
  }

  static Future<bool> show(
    BuildContext context, {
    required String title,
    required String description,
    required String cancelButtonText,
    required String confirmButtonText,
    Color? confirmButtonColor,
  }) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isScrollControlled: true,
      builder: (BuildContext context) {
        return ConfirmActionSheet(
          title: title,
          description: description,
          cancelButtonText: cancelButtonText,
          confirmButtonText: confirmButtonText,
          onConfirm: () => Navigator.pop(context, true),
          onCancel: () => Navigator.pop(context, false),
          confirmButtonColor: confirmButtonColor,
        );
      },
    );
    return confirmed == true;
  }

  ElevatedButtonThemeData? buttonTheme(BuildContext context) {
    if (confirmButtonColor == null) return null;
    return ElevatedButtonThemeData(
      style: ElevatedButtonTheme.of(context).style?.copyWith(
            backgroundColor: MaterialStatePropertyAll(confirmButtonColor!),
          ),
    );
  }
}
