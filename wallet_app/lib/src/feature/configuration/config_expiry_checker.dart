import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../domain/usecase/configuration/observe_configuration_expired_usecase.dart';
import '../../domain/usecase/wallet/lock_wallet_usecase.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/widget/minimal_wallet_app.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/utility/do_on_init.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_button_builder.dart';
import '../error/error_page.dart';

/// This widget observes whether the current wallet configuration has expired.
/// It intentionally lives above the [WalletApp] widget to make sure
/// no accidental navigation is possible while the configuration can't be trusted.
///
/// When expired, it locks the wallet, destroys the child widget tree (which includes
/// the MaterialApp) to block navigation, and shows the [_ConfigExpiredScreen].
/// Once the configuration is valid again, the child is restored.
class ConfigExpiryChecker extends StatelessWidget {
  final Widget child;

  const ConfigExpiryChecker({
    required this.child,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<bool>(
      stream: context.read<ObserveConfigurationExpiredUseCase>().invoke(),
      builder: (context, snapshot) {
        final isExpired = snapshot.data ?? true;
        if (!isExpired) return child;

        return DoOnInit(
          onInit: (BuildContext context) => context.read<LockWalletUseCase>().invoke(),
          child: const MinimalWalletApp(child: _ConfigExpiredScreen()),
        );
      },
    );
  }
}

/// Screen shown when the configuration is expired.
///
/// Restricted to only showing error details with no navigation or exit options,
/// ensuring the app remains unusable until a valid configuration is available.
class _ConfigExpiredScreen extends StatelessWidget {
  const _ConfigExpiredScreen();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.errorScreenServerHeadline),
        automaticallyImplyLeading: false,
      ),
      body: ErrorPage(
        title: context.l10n.errorScreenServerHeadline,
        description: context.l10n.errorScreenServerDescriptionCloseVariant,
        illustration: WalletAssets.svg_error_server_outage,
        primaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
      ),
    );
  }
}
