import 'package:flutter/material.dart';

import '../../../wallet_assets.dart';

class WalletLogo extends StatelessWidget {
  final double size;

  const WalletLogo({required this.size, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Image.asset(
      WalletAssets.logo_wallet,
      width: size,
      height: size,
    );
  }
}
