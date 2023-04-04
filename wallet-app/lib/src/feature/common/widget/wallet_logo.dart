import 'package:flutter/material.dart';

class WalletLogo extends StatelessWidget {
  final double size;

  const WalletLogo({required this.size, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Image.asset(
      'assets/non-free/images/logo_wallet.png',
      width: size,
      height: size,
    );
  }
}
