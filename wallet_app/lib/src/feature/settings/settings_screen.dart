import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/biometrics_extension.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/object_extension.dart';
import '../../util/extension/string_extension.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('settingsScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.settingsScreenTitle,
                      scrollController: PrimaryScrollController.maybeOf(context),
                    ),
                    _buildContentSliver(context),
                  ],
                ),
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    return SliverList.list(
      children: [
        const SizedBox(height: 16),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.settingsScreenChangePinCta.toTextSpan(context)),
          leftIcon: const Icon(Icons.key),
          onPressed: () => Navigator.pushNamed(context, WalletRoutes.changePinRoute),
        ),
        const Divider(),
        FutureBuilder<Biometrics>(
          future: context.read<GetSupportedBiometricsUseCase>().invoke(),
          initialData: Biometrics.some,
          builder: (context, snapshot) {
            final biometricsSupported = snapshot.data != Biometrics.none;
            final Biometrics biometrics = snapshot.data ?? Biometrics.some;
            return MenuItem(
              label: Text.rich(
                context.l10n.settingsScreenSetupBiometricsCta(biometrics.prettyPrint(context)).toTextSpan(context),
              ),
              subtitle: Text.rich(context.l10n.settingsScreenSetupBiometricsNotSupportedSubtitle.toTextSpan(context))
                  .takeIf((_) => !biometricsSupported),
              leftIcon: Icon(biometrics.icon),
              onPressed:
                  biometricsSupported ? () => Navigator.pushNamed(context, WalletRoutes.biometricsSettingsRoute) : null,
            );
          },
        ),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.settingsScreenChangeLanguageCta.toTextSpan(context)),
          leftIcon: const Icon(Icons.translate),
          onPressed: () => Navigator.pushNamed(context, WalletRoutes.changeLanguageRoute),
        ),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.settingsScreenClearDataCta.toTextSpan(context)),
          leftIcon: const Icon(Icons.delete_outline),
          onPressed: () => ResetWalletDialog.show(context),
        ),
        const Divider(),
        const SizedBox(height: 24),
      ],
    );
  }
}
