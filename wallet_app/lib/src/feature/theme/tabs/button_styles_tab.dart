import 'package:flutter/material.dart';

import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/destructive_button.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/text_with_link.dart';
import '../theme_screen.dart';

class ButtonStylesTab extends StatelessWidget {
  const ButtonStylesTab({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
      children: [
        const ThemeSectionSubHeader(title: 'Themed Framework Buttons'),
        const SizedBox(height: 16),
        ElevatedButton(
          onPressed: () => {},
          child: const Text('ElevatedButton'),
        ),
        const SizedBox(height: 16),
        OutlinedButton(
          onPressed: () => {},
          child: const Text('OutlinedButton'),
        ),
        const SizedBox(height: 16),
        TextButton(
          onPressed: () => {},
          child: const Text('TextButton'),
        ),
        const SizedBox(height: 16),
        const ThemeSectionSubHeader(title: 'Wallet Buttons'),
        const SizedBox(height: 16),
        PrimaryButton(onPressed: () => {}, text: const Text('Primary')),
        const SizedBox(height: 16),
        SecondaryButton(onPressed: () => {}, text: const Text('Secondary')),
        const SizedBox(height: 16),
        TertiaryButton(onPressed: () => {}, text: const Text('Tertiary')),
        const SizedBox(height: 16),
        DestructiveButton(onPressed: () => {}, text: const Text('Destructive')),
        const SizedBox(height: 16),
        const ThemeSectionSubHeader(title: 'TextWithLink'),
        TextWithLink(
          fullText: 'This is the full text {WITH} a clickable placeholder.',
          ctaText: 'WITH',
          onCtaPressed: () {},
        ),
        const ThemeSectionSubHeader(title: 'ListButton'),
        ListButton(
          onPressed: () => {},
          dividerSide: DividerSide.none,
          text: const Text('ListButton'),
        ),
        const ThemeSectionSubHeader(title: 'LinkButton'),
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: LinkButton(
            onPressed: () => {},
            text: const Text('LinkButton'),
          ),
        ),
        const ThemeSectionSubHeader(title: 'BottomBackButton'),
        const BottomBackButton(),
        const ThemeSectionSubHeader(title: 'ConfirmButtons'),
        const ConfirmButtons(
          primaryButton: PrimaryButton(
            key: Key('acceptButton'),
            text: Text('Accept'),
            icon: null,
          ),
          secondaryButton: SecondaryButton(
            key: Key('rejectButton'),
            icon: null,
            text: Text('Decline'),
          ),
        ),
      ],
    );
  }
}
