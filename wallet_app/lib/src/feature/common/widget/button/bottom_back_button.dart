import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import 'text_icon_button.dart';

const _kButtonHeight = 72.0;

class BottomBackButton extends StatelessWidget {
  const BottomBackButton({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final themeData = Theme.of(context);
    return Align(
      alignment: Alignment.bottomCenter,
      child: SizedBox(
        height: _kButtonHeight,
        width: double.infinity,
        child: Theme(
          data: themeData.copyWith(
            textButtonTheme: TextButtonThemeData(
              style: themeData.textButtonTheme.style?.copyWith(
                // Remove rounded edges
                shape: const MaterialStatePropertyAll(RoundedRectangleBorder()),
              ),
            ),
          ),
          child: TextIconButton(
            onPressed: () => Navigator.pop(context),
            iconPosition: IconPosition.start,
            icon: Icons.arrow_back,
            child: Text(AppLocalizations.of(context).generalBottomBackCta),
          ),
        ),
      ),
    );
  }
}
