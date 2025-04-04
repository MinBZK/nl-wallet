import 'package:flutter/cupertino.dart';

import '../../../theme/wallet_theme.dart';

class CardShadowDecoration extends Decoration {
  static const _decoration = BoxDecoration(
    borderRadius: WalletTheme.kBorderRadius12,
    boxShadow: [
      BoxShadow(
        color: Color(0x0000000D),
        blurRadius: 15,
        offset: Offset(0, 1),
      ),
      BoxShadow(
        color: Color(0x152A621A),
        blurRadius: 4,
        offset: Offset(0, 4),
      ),
    ],
  );

  const CardShadowDecoration();

  @override
  BoxPainter createBoxPainter([VoidCallback? onChanged]) => _decoration.createBoxPainter(onChanged);
}
