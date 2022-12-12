import 'package:flutter/material.dart';

class SignPolicyRow extends StatelessWidget {
  final IconData icon;
  final String title;

  const SignPolicyRow({
    required this.icon,
    required this.title,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        SizedBox(
          height: 56,
          width: 56,
          child: Icon(icon),
        ),
        Expanded(
          child: Text(title, style: Theme.of(context).textTheme.bodyText1),
        ),
        const SizedBox(width: 16),
      ],
    );
  }
}
