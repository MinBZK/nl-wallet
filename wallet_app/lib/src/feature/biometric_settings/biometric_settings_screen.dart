import 'dart:io';

import 'package:app_settings/app_settings.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/biometrics/biometrics.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/biometrics_extension.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/locked_out_dialog.dart';
import '../common/page/page_illustration.dart';
import '../common/screen/confirm_with_pin_screen.dart';
import '../common/screen/terminal_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/paragraphed_sliver_list.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../error/error_screen.dart';
import 'bloc/biometric_settings_bloc.dart';

class BiometricSettingScreen extends StatelessWidget {
  const BiometricSettingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: BlocConsumer<BiometricSettingsBloc, BiometricSettingsState>(
                buildWhen: (prev, current) {
                  return switch (current) {
                    BiometricSettingsConfirmPin() => false,
                    BiometricSettingsSetupRequired() => false,
                    BiometricSettingsLockedOut() => false,
                    _ => true,
                  };
                },
                listenWhen: (prev, current) {
                  return switch (current) {
                    BiometricSettingsConfirmPin() => true,
                    BiometricSettingsSetupRequired() => true,
                    BiometricSettingsLockedOut() => true,
                    _ => false,
                  };
                },
                builder: (context, state) {
                  assert(state is! BiometricSettingsConfirmPin, 'BiometricSettingsConfirmPin should never be rendered');
                  assert(
                    state is! BiometricSettingsSetupRequired,
                    'BiometricSettingsSetupRequired should never be rendered',
                  );
                  assert(state is! BiometricSettingsLockedOut, 'BiometricSettingsLockedOut should never be rendered');
                  return switch (state) {
                    BiometricSettingsLoaded() => _buildLoaded(context, state),
                    BiometricSettingsError() => _buildError(context, state),
                    _ => _buildLoading(context),
                  };
                },
                listener: (BuildContext context, BiometricSettingsState state) async {
                  final bloc = context.bloc;
                  if (state is BiometricSettingsConfirmPin) {
                    await _onRequestConfirmPin(context);
                    // Refresh state, relevant when confirmation failed.
                    bloc.add(const BiometricLoadTriggered());
                  } else if (state is BiometricSettingsSetupRequired) {
                    await _showSetupRequiredDialog(context);
                  } else if (state is BiometricSettingsLockedOut) {
                    await LockedOutDialog.show(context);
                  }
                },
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Future<void> _onRequestConfirmPin(BuildContext context) async {
    final bloc = context.bloc;
    final supportedBiometricsText = bloc.supportedBiometrics.prettyPrint(context);
    final illustration = switch (bloc.supportedBiometrics) {
      Biometrics.face => WalletAssets.svg_biometrics_face,
      Biometrics.fingerprint => WalletAssets.svg_biometrics_finger,
      _ => Platform.isIOS ? WalletAssets.svg_biometrics_face : WalletAssets.svg_biometrics_finger,
    };
    return ConfirmWithPinScreen.show(
      context,
      (_) {
        // Pin validated! Confirm settings update.
        bloc.add(const BiometricUnlockEnabledWithPin());
        // Replace pin confirmation screen with a success screen
        TerminalScreen.show(
          context,
          replaceCurrentRoute: true,
          config: TerminalScreenConfig(
            title: context.l10n.biometricSettingsScreenSuccessTitle,
            description: context.l10n.biometricSettingsScreenSuccessDescription(supportedBiometricsText),
            illustration: illustration,
            secondaryButton: TertiaryButton(
              text: Text(context.l10n.biometricSettingsScreenSuccessToSettingsCta),
              onPressed: () {
                Navigator.popUntil(
                  context,
                  ModalRoute.withName(WalletRoutes.settingsRoute),
                );
              },
            ),
          ),
        );
      },
    );
  }

  Future<void> _showSetupRequiredDialog(BuildContext context) async {
    final supportedBiometricsText = context.bloc.supportedBiometrics.prettyPrint(context);
    final title = context.l10n.biometricSettingsScreenSetupDialogTitle(supportedBiometricsText);

    return showDialog<void>(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          scrollable: true,
          semanticLabel: title,
          title: Text(title),
          content: Text(context.l10n.biometricSettingsScreenSetupDialogDescription(supportedBiometricsText)),
          actions: <Widget>[
            TextButton(
              child: Text(context.l10n.generalDialogCloseCta),
              onPressed: () => Navigator.of(context).pop(),
            ),
            TextButton(
              child: Text(context.l10n.biometricSettingsScreenSetupDialogOpenSettingsCta),
              onPressed: () {
                // NOTE: Plugins to open biometric settings seem flaky (i.e. don't work on my Pixel 6 Pro),
                // NOTE: we could likely roll our own but falling back to generic settings for now.
                // NOTE: Also note that this dialog is already a fallback, normally a system dialog should
                // NOTE: redirect the user to the correct place. This dialog can however be triggered on e.g.
                // NOTE: a Pixel 6 Pro by enrolling fingerprints but disabling 'allow apps to verify your identity'
                // NOTE: in the device's biometric settings.
                AppSettings.openAppSettings(type: AppSettingsType.device, asAnotherTask: true);
                Navigator.of(context).pop();
              },
            ),
          ],
        );
      },
    );
  }

  Widget _buildLoading(BuildContext context) {
    return CustomScrollView(
      slivers: [
        SliverWalletAppBar(title: _resolveTitle(context)),
        const SliverFillRemaining(
          child: CenteredLoadingIndicator(),
        ),
      ],
    );
  }

  Widget _buildLoaded(BuildContext context, BiometricSettingsLoaded state) {
    final inactiveThumbColor = context.colorScheme.onSurfaceVariant;
    final WidgetStateProperty<Icon?> thumbIcon = WidgetStateProperty.resolveWith<Icon?>(
      (Set<WidgetState> states) {
        if (states.contains(WidgetState.selected)) return null;
        // There is no property to change the size of the 'inactive' thumb, but it does grow
        // when an icon is specified, as such we provide an empty icon here.
        return const Icon(null);
      },
    );
    final supportedBiometricsText = context.bloc.supportedBiometrics.prettyPrint(context);
    return CustomScrollView(
      slivers: [
        SliverWalletAppBar(title: _resolveTitle(context)),
        SliverPadding(
          padding: const EdgeInsets.all(16),
          sliver: ParagraphedSliverList.splitContent(
            context.l10n.biometricSettingsScreenDescription(supportedBiometricsText),
            splitPattern: '\n',
          ),
        ),
        SliverToBoxAdapter(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Divider(height: 16),
              SwitchListTile(
                value: state.biometricLoginEnabled,
                inactiveThumbColor: inactiveThumbColor,
                thumbIcon: thumbIcon,
                trackOutlineColor: state.biometricLoginEnabled ? null : WidgetStatePropertyAll(inactiveThumbColor),
                onChanged: (enabled) => context.bloc.add(const BiometricUnlockToggled()),
                title: Text.rich(
                  context.l10n.biometricSettingsScreenSwitchCta(supportedBiometricsText).toTextSpan(context),
                  style: context.textTheme.labelLarge,
                ),
              ),
              const Divider(height: 16),
            ],
          ),
        ),
        const SliverSizedBox(height: 16),
        const SliverToBoxAdapter(
          child: PageIllustration(asset: WalletAssets.svg_biometrics_face),
        ),
        const SliverSizedBox(height: 24),
      ],
    );
  }

  Widget _buildError(BuildContext context, BiometricSettingsError state) {
    return ErrorScreen(
      title: context.l10n.errorScreenGenericHeadline,
      description: context.l10n.errorScreenGenericDescription,
      primaryButton: PrimaryButton(
        text: Text(context.l10n.generalRetry),
        onPressed: () => context.bloc.add(const BiometricLoadTriggered()),
      ),
    );
  }

  String _resolveTitle(BuildContext context) =>
      context.l10n.biometricSettingsScreenTitle(context.bloc.supportedBiometrics.prettyPrint(context));
}

extension _BiometricSettingsScreenExtensions on BuildContext {
  BiometricSettingsBloc get bloc => read<BiometricSettingsBloc>();
}
