import 'package:flutter/material.dart';

import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/button/link_tile_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/rounded_back_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/button/text_icon_button.dart';
import '../../common/widget/text_with_link.dart';
import '../theme_screen.dart';

class ButtonStylesTab extends StatelessWidget {
  const ButtonStylesTab({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 16),
      children: [
        const ThemeSectionSubHeader(title: 'Themed Framework Buttons'),
        const SizedBox(height: 16),
        ElevatedButton(
          onPressed: () => {},
          child: const Text('ElevatedButton'),
        ),
        const SizedBox(height: 16),
        TextButton(
          onPressed: () => {},
          child: const Text('TextButton'),
        ),
        const SizedBox(height: 16),
        OutlinedButton(
          onPressed: () => {},
          child: const Text('OutlinedButton'),
        ),
        const SizedBox(height: 16),
        const ThemeSectionSubHeader(title: 'Primary & Secondary Buttons'),
        const SizedBox(height: 16),
        PrimaryButton(onPressed: () => {}, text: 'PrimaryButton'),
        const SizedBox(height: 16),
        SecondaryButton(onPressed: () => {}, text: 'SecondaryButton'),
        const SizedBox(height: 16),
        const ThemeSectionSubHeader(title: 'TextIconButton'),
        TextIconButton(
          onPressed: () => {},
          child: const Text('TextIconButton'),
        ),
        const ThemeSectionSubHeader(title: 'TextWithLink'),
        TextWithLink(
          fullText: 'This is the full text {WITH} a clickable placeholder.',
          ctaText: 'WITH',
          onCtaPressed: () {},
        ),
        const ThemeSectionSubHeader(title: 'LinkTileButton'),
        LinkTileButton(
          onPressed: () => {},
          child: const Text('LinkTileButton'),
        ),
        const ThemeSectionSubHeader(title: 'LinkButton'),
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: LinkButton(
            onPressed: () => {},
            child: const Text('LinkButton'),
          ),
        ),
        const SizedBox(height: 16),
        const ThemeSectionSubHeader(title: 'BottomBackButton'),
        const BottomBackButton(),
        const ThemeSectionSubHeader(title: 'RoundedBackButton'),
        const RoundedBackButton(),
        const ThemeSectionSubHeader(title: 'ConfirmButtons'),
        ConfirmButtons(
            onSecondaryPressed: () {}, onPrimaryPressed: () {}, primaryText: 'Accept', secondaryText: 'Decline'),
      ],
    );
  }
}
