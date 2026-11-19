import 'dart:io';

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:material_ui/material_ui.dart';

import '../../domain/usecase/biometrics/biometrics.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/biometrics_extension.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/dialog/biometrics_setup_required_dialog.dart';
import '../common/dialog/locked_out_dialog.dart';
import '../common/screen/confirm_with_pin_screen.dart';
import '../common/screen/terminal_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/paragraphed_sliver_list.dart';
import '../common/widget/setting/switch_setting_row.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_screen.dart';
import 'bloc/biometric_settings_bloc.dart';

class BiometricSettingScreen extends StatelessWidget {
  const BiometricSettingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(_resolveTitle(context)),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildContent(context),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return BlocConsumer<BiometricSettingsBloc, BiometricSettingsState>(
      listenWhen: _listenWhenForState,
      listener: _listenerForState,
      buildWhen: _buildWhenForState,
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
    );
  }

  bool _listenWhenForState(BiometricSettingsState prevState, BiometricSettingsState currentState) {
    return switch (currentState) {
      BiometricSettingsConfirmPin() => true,
      BiometricSettingsSetupRequired() => true,
      BiometricSettingsLockedOut() => true,
      _ => false,
    };
  }

  Future<void> _listenerForState(BuildContext context, BiometricSettingsState state) async {
    final bloc = context.bloc;
    if (state is BiometricSettingsConfirmPin) {
      await _onRequestConfirmPin(context);
      // Refresh state, relevant when confirmation failed.
      bloc.add(const BiometricLoadTriggered());
    } else if (state is BiometricSettingsSetupRequired) {
      await BiometricsSetupRequiredDialog.show(context, bloc.supportedBiometrics);
    } else if (state is BiometricSettingsLockedOut) {
      await LockedOutDialog.show(context);
    }
  }

  bool _buildWhenForState(BiometricSettingsState prevState, BiometricSettingsState currentState) {
    return switch (currentState) {
      BiometricSettingsConfirmPin() => false,
      BiometricSettingsSetupRequired() => false,
      BiometricSettingsLockedOut() => false,
      _ => true,
    };
  }

  Widget _buildLoading(BuildContext context) {
    return CustomScrollView(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: kDefaultTitlePadding,
            child: TitleText(_resolveTitle(context)),
          ),
        ),
        const SliverFillRemaining(
          child: CenteredLoadingIndicator(),
        ),
      ],
    );
  }

  Widget _buildLoaded(BuildContext context, BiometricSettingsLoaded state) {
    final supportedBiometricsText = context.bloc.supportedBiometrics.prettyPrint(context);
    return CustomScrollView(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: kDefaultTitlePadding,
            child: TitleText(_resolveTitle(context)),
          ),
        ),
        SliverPadding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          sliver: ParagraphedSliverList.splitContent(
            context.l10n.biometricSettingsScreenDescription(supportedBiometricsText),
          ),
        ),
        const SliverSizedBox(height: 24),
        SliverToBoxAdapter(
          child: SwitchSettingRow(
            dividerSide: .both,
            label: Text.rich(
              context.l10n.biometricSettingsScreenSwitchCta(supportedBiometricsText).toTextSpan(context),
            ),
            value: state.biometricLoginEnabled,
            onChanged: (enabled) => context.bloc.add(const BiometricUnlockToggled()),
          ),
        ),
        const SliverSizedBox(height: 24),
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
        text: Text.rich(context.l10n.generalRetry.toTextSpan(context)),
        onPressed: () => context.bloc.add(const BiometricLoadTriggered()),
      ),
    );
  }

  String _resolveTitle(BuildContext context) =>
      context.l10n.biometricSettingsScreenTitle(context.bloc.supportedBiometrics.prettyPrint(context));

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
              text: Text.rich(context.l10n.biometricSettingsScreenSuccessToSettingsCta.toTextSpan(context)),
              onPressed: () {
                Navigator.popUntil(
                  context,
                  ModalRoute.withName(WalletRoutes.settingsRoute),
                );
              },
            ),
            preferVerticalButtonLayout: false,
          ),
        );
      },
    );
  }
}

extension _BiometricSettingsScreenExtensions on BuildContext {
  BiometricSettingsBloc get bloc => read<BiometricSettingsBloc>();
}
