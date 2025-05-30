import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../environment.dart';
import '../../../data/service/navigation_service.dart';
import '../../../domain/model/navigation/navigation_request.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/spacer/paragraph_spacer.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../theme_screen.dart';

class TextStylesTab extends StatelessWidget {
  const TextStylesTab({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      children: [
        // Display
        Text('DisplayLarge', style: context.textTheme.displayLarge),
        Text('DisplayMedium', style: context.textTheme.displayMedium),
        Text('DisplaySmall', style: context.textTheme.displaySmall),
        // Headline
        Text('HeadlineLarge', style: context.textTheme.headlineLarge),
        Text('HeadlineMedium', style: context.textTheme.headlineMedium),
        Text('HeadlineSmall', style: context.textTheme.headlineSmall),
        // Title
        Text('TitleLarge', style: context.textTheme.titleLarge),
        Text('TitleMedium', style: context.textTheme.titleMedium),
        Text('TitleSmall', style: context.textTheme.titleSmall),
        // Body
        Text('BodyLarge', style: context.textTheme.bodyLarge),
        Text('BodyMedium', style: context.textTheme.bodyMedium),
        Text('BodySmall', style: context.textTheme.bodySmall),
        // Label
        Text('LabelLarge', style: context.textTheme.labelLarge),
        Text('LabelMedium', style: context.textTheme.labelMedium),
        Text('LabelSmall', style: context.textTheme.labelSmall),
        const Divider(height: 24),
        const TitleText('Title Text'),
        const ParagraphSpacer(),
        const BodyText('Body Text'),
        _buildScenarios(context),
      ],
    );
  }

  Widget _buildScenarios(BuildContext context) {
    if (!Environment.mockRepositories) return const SizedBox.shrink();
    return Column(
      children: [
        const ThemeSectionHeader(title: 'Scenarios'),
        const SizedBox(height: 12),
        SecondaryButton(
          onPressed: () {
            final request = DisclosureNavigationRequest(
              'walletdebuginteraction://deeplink#%7B%22id%22%3A%22OPEN_BANK_ACCOUNT%22%2C%22type%22%3A%22verify%22%7D',
            );
            context.read<NavigationService>().handleNavigationRequest(request);
          },
          icon: const Icon(Icons.share_outlined),
          text: const Text('disclose'),
        ),
        const SizedBox(height: 12),
        SecondaryButton(
          onPressed: () {
            final request = DisclosureNavigationRequest(
              'walletdebuginteraction://deeplink#%7B%22id%22%3A%22AMSTERDAM_LOGIN%22%2C%22type%22%3A%22verify%22%7D',
            );
            context.read<NavigationService>().handleNavigationRequest(request);
          },
          icon: const Icon(Icons.login_outlined),
          text: const Text('login'),
        ),
        const SizedBox(height: 12),
        SecondaryButton(
          onPressed: () {
            final request = DisclosureNavigationRequest(
              'walletdebuginteraction://deeplink#%7B%22id%22%3A%22CAR_RENTAL%22%2C%22type%22%3A%22verify%22%7D',
            );
            context.read<NavigationService>().handleNavigationRequest(request);
          },
          icon: const Icon(Icons.warning_amber_outlined),
          text: const Text('missing attributes'),
        ),
        const SizedBox(height: 12),
        SecondaryButton(
          onPressed: () {
            final request = IssuanceNavigationRequest(
              'walletdebuginteraction://deeplink#%7B%22id%22%3A%22DRIVING_LICENSE%22%2C%22type%22%3A%22issue%22%7D',
            );
            context.read<NavigationService>().handleNavigationRequest(request);
          },
          icon: const Icon(Icons.credit_card_outlined),
          text: const Text('issue'),
        ),
        const SizedBox(height: 12),
        SecondaryButton(
          onPressed: () {
            final request = SignNavigationRequest(
              'walletdebuginteraction://deeplink#%7B%22id%22%3A%22RENTAL_AGREEMENT%22%2C%22type%22%3A%22sign%22%7D',
            );
            context.read<NavigationService>().handleNavigationRequest(request);
          },
          icon: const Icon(Icons.document_scanner_outlined),
          text: const Text('sign'),
        ),
      ],
    );
  }
}
