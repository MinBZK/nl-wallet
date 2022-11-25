import 'package:flutter/material.dart';

class ConfirmButtons extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final String acceptText;
  final String declineText;
  final IconData? acceptIcon;
  final IconData? declineIcon;

  const ConfirmButtons({
    required this.onDecline,
    required this.onAccept,
    required this.acceptText,
    required this.declineText,
    this.acceptIcon = Icons.check,
    this.declineIcon = Icons.not_interested,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Row(
        mainAxisSize: MainAxisSize.max,
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          Expanded(
            child: SizedBox(
              height: 48,
              child: OutlinedButton(
                onPressed: onDecline,
                child: declineIcon == null
                    ? Text(declineText)
                    : Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(declineIcon, size: 16),
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
                child: acceptIcon == null
                    ? Text(acceptText)
                    : Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(acceptIcon, size: 16),
                          const SizedBox(width: 8),
                          Text(acceptText),
                        ],
                      ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
