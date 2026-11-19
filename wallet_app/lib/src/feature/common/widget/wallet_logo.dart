import 'package:material_ui/material_ui.dart';

import '../../../wallet_assets.dart';

class WalletLogo extends StatelessWidget {
  final double size;

  const WalletLogo({required this.size, super.key});

  @override
  Widget build(BuildContext context) {
    return Image.asset(
      WalletAssets.logo_wallet,
      width: size,
      height: size,
    );
  }
}
