import 'package:flutter/material.dart';

class ConfirmButtons extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final String acceptText;
  final String declineText;

  const ConfirmButtons({
    required this.onDecline,
    required this.onAccept,
    required this.acceptText,
    required this.declineText,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Align(
        alignment: Alignment.bottomCenter,
        child: Row(
          mainAxisSize: MainAxisSize.max,
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            Expanded(
              child: SizedBox(
                height: 48,
                child: OutlinedButton(
                  onPressed: onDecline,
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(
                        Icons.not_interested,
                        size: 16,
                      ),
                      const SizedBox(width: 8),
                      Text(declineText),
                    ],
                  ),
                ),
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: SizedBox(
                height: 48,
                child: ElevatedButton(
                  onPressed: onAccept,
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(
                        Icons.check,
                        size: 16,
                      ),
                      const SizedBox(width: 8),
                      Text(acceptText),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
